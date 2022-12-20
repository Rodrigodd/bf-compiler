use std::process::ExitCode;

use dynasmrt::{dynasm, x64::X64Relocation, DynasmApi, DynasmLabelApi, VecAssembler};

use object::{
    elf::{
        ELFOSABI_SYSV, EM_X86_64, ET_EXEC, PF_R, PF_X, PT_LOAD, SHF_ALLOC, SHF_EXECINSTR,
        SHT_PROGBITS,
    },
    write::{
        elf::{FileHeader, ProgramHeader},
        Symbol,
    },
    SymbolFlags,
};

struct UnbalancedBrackets(char, usize);

struct Program {
    code: Vec<u8>,
}
impl Program {
    fn new(source: &[u8]) -> Result<Program, UnbalancedBrackets> {
        let mut code: VecAssembler<X64Relocation> = VecAssembler::new(0);

        // r12 will be the adress of `memory`
        // r13 will be the value of `pointer`
        // r13 is set to 0
        dynasm! { code
            ; .arch x64
            ; push rbp
            ; mov rbp, rsp
            ; xor r13, r13
            // allocate 30_0000 bytes on stack for the memory
            ; sub rsp, 30_000
            ; mov r12, rsp
        };

        let mut bracket_stack = Vec::new();

        for b in source {
            match b {
                b'+' => dynasm! { code
                    ; .arch x64
                    ; add BYTE [r12 + r13], 1
                },
                b'-' => dynasm! { code
                    ; .arch x64
                    ; add BYTE [r12 + r13], -1
                },
                b'.' => dynasm! { code
                    ; .arch x64
                    ; mov rax, 1 // write syscall
                    ; mov rdi, 1 // stdout's file descriptor
                    ; lea rsi, [r12 + r13] // buf address
                    ; mov rdx, 1           // length
                    ; syscall
                },
                b',' => dynasm! { code
                    ; .arch x64
                    ; mov rax, 0 // read syscall
                    ; mov rdi, 0 // stdin's file descriptor
                    ; lea rsi, [r12 + r13] // buf address
                    ; mov rdx, 1           // length
                    ; syscall
                },
                b'<' => dynasm! { code
                    ; .arch x64
                    ; sub r13, 1
                    ; mov eax, 29999
                    ; cmovb r13, rax
                },
                b'>' => dynasm! { code
                    ; .arch x64
                    ; add r13, 1
                    ; xor eax, eax
                    ; cmp r13, 30000
                    ; cmove r13, rax
                },
                b'[' => {
                    let start_label = code.new_dynamic_label();
                    let end_label = code.new_dynamic_label();
                    dynasm! { code
                        ; .arch x64
                        ; cmp BYTE [r12+r13], 0
                        ; je =>end_label
                        ; =>start_label
                    };

                    bracket_stack.push((start_label, end_label));
                }
                b']' => {
                    let (start_label, end_label) = match bracket_stack.pop() {
                        Some(x) => x,
                        None => return Err(UnbalancedBrackets(']', code.offset().0)),
                    };

                    dynasm! { code
                        ; .arch x64
                        ; cmp BYTE [r12 + r13], 0
                        ; jne =>start_label
                        ; => end_label
                    };
                }
                _ => continue,
            }
        }

        if !bracket_stack.is_empty() {
            return Err(UnbalancedBrackets(']', code.offset().0));
        }

        dynasm! { code
            ; .arch x64
            ; xor rax, rax
            ; ->exit:
            ; mov rdi, rax // exit error code
            ; mov rax, 60 // exit syscall
            ; syscall
        }

        Ok(Program {
            code: code.finalize().unwrap(),
        })
    }

    fn to_elf_object(&self) -> Vec<u8> {
        let mut obj = object::write::Object::new(
            object::BinaryFormat::Elf,
            object::Architecture::X86_64,
            object::Endianness::Little,
        );

        let start = obj.add_symbol(Symbol {
            name: b"_start".to_vec(),
            value: 0,
            size: 0,
            kind: object::SymbolKind::Text,
            scope: object::SymbolScope::Linkage,
            weak: false,
            section: object::write::SymbolSection::Undefined,
            flags: SymbolFlags::None,
        });

        let text = obj.section_id(object::write::StandardSection::Text);
        obj.add_symbol_data(start, text, &self.code, 16);
        let mut out = Vec::new();
        obj.emit(&mut out).unwrap();

        out
    }

    fn to_elf_executable(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut writer =
            object::write::elf::Writer::new(object::Endianness::Little, true, &mut out);

        let text_name = writer.add_section_name(b".text");
        let _text_section = writer.reserve_section_index();

        writer.reserve_file_header();

        writer.reserve_program_headers(1);

        writer.reserve_strtab_section_index();
        writer.reserve_strtab();

        writer.reserve_shstrtab_section_index();
        writer.reserve_shstrtab();

        writer.reserve_section_headers();

        let text_offset = writer.reserve(self.code.len(), 16);

        const PAGE_SIZE: u64 = 0x1000;

        writer
            .write_file_header(&FileHeader {
                os_abi: ELFOSABI_SYSV,
                abi_version: 0,
                e_type: ET_EXEC,
                e_machine: EM_X86_64,
                e_entry: 0x400000 + (text_offset as u64 % PAGE_SIZE),
                e_flags: 0,
            })
            .unwrap();

        writer.write_align_program_headers();
        writer.write_program_header(&ProgramHeader {
            p_type: PT_LOAD,
            p_flags: PF_R | PF_X,
            p_offset: text_offset as u64,
            p_vaddr: 0x400000 + (text_offset as u64 % PAGE_SIZE),
            p_paddr: 0,
            p_filesz: self.code.len() as u64,
            p_memsz: self.code.len() as u64,
            p_align: PAGE_SIZE,
        });

        writer.write_strtab();

        writer.write_shstrtab();

        writer.write_null_section_header();

        writer.write_section_header(&object::write::elf::SectionHeader {
            name: Some(text_name),
            sh_type: SHT_PROGBITS,
            sh_flags: (SHF_ALLOC | SHF_EXECINSTR) as u64,
            sh_addr: 0x400000,
            sh_offset: text_offset as u64,
            sh_size: self.code.len() as u64,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 16,
            sh_entsize: 0,
        });

        writer.write_strtab_section_header();
        writer.write_shstrtab_section_header();

        writer.write_align(16);
        writer.write(&self.code);

        assert_eq!(writer.reserved_len(), writer.len());

        out
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args();

    let file_name = args.nth(1).unwrap();
    let source = match std::fs::read(&file_name) {
        Ok(x) => x,
        Err(err) => {
            eprintln!("Error reading '{}': {}", file_name, err);
            return ExitCode::from(2);
        }
    };

    let program = match Program::new(&source) {
        Ok(x) => x,
        Err(UnbalancedBrackets(c, address)) => {
            eprintln!(
                "Error parsing file: didn't found pair for `{}` at instruction index {}",
                c, address
            );
            return ExitCode::from(3);
        }
    };

    let option = args.next();
    let output_name = args.next().unwrap_or_else(|| {
        std::path::Path::new(&file_name)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
    });
    match option.unwrap().as_str() {
        "-o" => {
            let output_name = std::path::Path::new(&output_name).with_extension("o");
            let obj = program.to_elf_object();
            std::fs::write(output_name, obj).unwrap();
        }
        "-x" => {
            let output_name = std::path::Path::new(&output_name).with_extension("");
            let exe = program.to_elf_executable();
            std::fs::write(output_name, exe).unwrap();
        }
        arg => panic!("unknown arg {arg}"),
    }

    ExitCode::from(0)
}
