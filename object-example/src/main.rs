use dynasmrt::x64::X64Relocation;
use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi, VecAssembler};

use object::{
    elf::{
        ELFOSABI_SYSV, EM_X86_64, ET_EXEC, PF_R, PF_X, PT_LOAD, SHF_ALLOC, SHF_EXECINSTR,
        SHT_PROGBITS,
    },
    write::{
        elf::{FileHeader, ProgramHeader},
        Relocation, Symbol,
    },
    SymbolFlags,
};

fn main() {
    let mut code: VecAssembler<X64Relocation> = VecAssembler::new(0);

    let hello_str = b"Hello world!\n";
    let len = hello_str.len() as i32;

    let relocation_offset;
    dynasm!(code
        ; lea rdi, [>hello]    // string to write
        ; mov rsi, DWORD len   // length of string to write
        ; call DWORD 0
        ;; relocation_offset = code.offset().0 as u64 - 4

        // Terminate program
        ; mov eax,60           // 'exit' system call
        ; mov edi,0            // exit with error code 0
        ; syscall              // call the kernel
        ; hello:
        ; .bytes hello_str
    );

    let code = code.finalize().unwrap();

    let mut buffer = memmap2::MmapOptions::new()
        .len(code.len())
        .map_anon()
        .unwrap();

    buffer.copy_from_slice(code.as_slice());

    let buffer = buffer.make_exec().unwrap();

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("expected 1 argument");
        return;
    }
    match args[1].as_str() {
        "run" => unsafe {
            let hello: unsafe extern "C" fn() -> ! = std::mem::transmute(buffer.as_ptr());
            hello()
        },
        "obj" => {
            let mut obj = object::write::Object::new(
                object::BinaryFormat::Elf,
                object::Architecture::X86_64,
                object::Endianness::Little,
            );

            let start = obj.add_symbol(Symbol {
                name: b"_start".to_vec(),
                kind: object::SymbolKind::Text,
                scope: object::SymbolScope::Linkage,
                weak: false,
                flags: SymbolFlags::None,

                value: 0,
                size: 0,
                section: object::write::SymbolSection::Undefined,
            });
            let my_write = obj.add_symbol(Symbol {
                name: b"my_write".to_vec(),
                kind: object::SymbolKind::Text,
                scope: object::SymbolScope::Linkage,
                weak: false,
                flags: SymbolFlags::None,

                value: 0,
                size: 0,
                section: object::write::SymbolSection::Undefined,
            });

            let text = obj.section_id(object::write::StandardSection::Text);
            obj.add_relocation(
                text,
                Relocation {
                    offset: relocation_offset,
                    size: 32,
                    kind: object::RelocationKind::Relative,
                    encoding: object::RelocationEncoding::Generic,
                    symbol: my_write,
                    addend: -4,
                },
            )
            .unwrap();

            obj.add_symbol_data(start, text, &code, 1);

            let mut out = Vec::new();
            obj.emit(&mut out).unwrap();

            std::fs::write("hello.o", out).unwrap();
        }
        "exe" => {
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

            let text_offset = writer.reserve(code.len(), 16);

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
                p_filesz: code.len() as u64,
                p_memsz: code.len() as u64,
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
                sh_size: code.len() as u64,
                sh_link: 0,
                sh_info: 0,
                sh_addralign: 16,
                sh_entsize: 0,
            });

            writer.write_strtab_section_header();
            writer.write_shstrtab_section_header();

            writer.write_align(16);
            writer.write(&code);

            assert_eq!(writer.reserved_len(), writer.len());

            std::fs::write("out.exe", out).unwrap();
        }
        _ => {
            eprintln!("unkown argument");
        }
    }
}
