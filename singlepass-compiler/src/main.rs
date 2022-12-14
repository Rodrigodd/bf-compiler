use std::process::ExitCode;

use dynasmrt::{dynasm, x64::X64Relocation, DynasmApi, DynasmLabelApi, VecAssembler};

use object::{
    write::{Relocation, Symbol},
    SymbolFlags,
};

struct UnbalancedBrackets(char, usize);

struct Program {
    code: Vec<u8>,
    write_relocations: Vec<usize>,
    read_relocations: Vec<usize>,
    exit_relocation: usize,
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

            // zero the memory
            ; xor eax, eax
            ; mov r11, rbp
            ; loop_:
            ; add r11, -8
            ; mov QWORD [r11], rax
            ; cmp r11, r12
            ; jne <loop_
        };

        let mut write_relocations = Vec::new();
        let mut read_relocations = Vec::new();

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
                    ; mov rdi, [r12 + r13] // cell value
                    ; call DWORD 0
                    ;; write_relocations.push(code.offset().0 - 4)
                },
                b',' => dynasm! { code
                    ; .arch x64
                    ; lea rdi, [r12 + r13] // cell address
                    ; call DWORD 0
                    ;; read_relocations.push(code.offset().0 - 4)
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

        let exit_relocation;

        dynasm! { code
            ; .arch x64
            ; call DWORD 0
            ;; exit_relocation = code.offset().0 - 4
        }

        Ok(Program {
            code: code.finalize().unwrap(),
            write_relocations,
            read_relocations,
            exit_relocation,
        })
    }

    fn to_elf_object(&self) -> Vec<u8> {
        let (format, entry_name) = if cfg!(target_os = "windows") {
            (object::BinaryFormat::Coff, "WinMain")
        } else if cfg!(target_os = "linux") {
            (object::BinaryFormat::Elf, "_start")
        } else {
            unimplemented!("Only Linux and Windows are implemented")
        };
        let entry_name = entry_name.as_bytes();

        let mut obj = object::write::Object::new(
            format,
            object::Architecture::X86_64,
            object::Endianness::Little,
        );

        let mut add_symbol = |name: &[u8]| {
            obj.add_symbol(Symbol {
                name: name.to_vec(),
                value: 0,
                size: 0,
                kind: object::SymbolKind::Text,
                scope: object::SymbolScope::Linkage,
                weak: false,
                section: object::write::SymbolSection::Undefined,
                flags: SymbolFlags::None,
            })
        };

        let start = add_symbol(entry_name);
        let bf_write = add_symbol(b"bf_write");
        let bf_read = add_symbol(b"bf_read");
        let bf_exit = add_symbol(b"bf_exit");

        let text = obj.section_id(object::write::StandardSection::Text);
        obj.add_symbol_data(start, text, &self.code, 16);

        let mut add_call_reloc = |offset, symbol| {
            obj.add_relocation(
                text,
                Relocation {
                    offset: offset as u64,
                    symbol,
                    size: 32,
                    kind: object::RelocationKind::Relative,
                    encoding: object::RelocationEncoding::Generic,
                    addend: -4,
                },
            )
            .unwrap();
        };

        for offset in self.read_relocations.iter().copied() {
            add_call_reloc(offset, bf_read);
        }
        for offset in self.write_relocations.iter().copied() {
            add_call_reloc(offset, bf_write);
        }
        add_call_reloc(self.exit_relocation, bf_exit);

        let mut out = Vec::new();
        obj.emit(&mut out).unwrap();

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
        arg => panic!("unknown arg {arg}"),
    }

    ExitCode::from(0)
}
