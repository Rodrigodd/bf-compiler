use cranelift::{
    codegen::{
        entity::EntityRef,
        ir::{
            condcodes::IntCC, types::I8, AbiParam, Function, InstBuilder, MemFlags, Signature,
            UserFuncName,
        },
        isa::{self, CallConv},
        settings::{self, Configurable},
        verify_function, Context,
    },
    frontend::{FunctionBuilder, FunctionBuilderContext, Variable},
};
use std::{
    io::{Read, Write},
    process::ExitCode,
};
use target_lexicon::Triple;

struct UnbalancedBrackets(char, usize);

struct Program {
    code: Vec<u8>,
    memory: [u8; 30_000],
}
impl Program {
    fn new(source: &[u8], clir: bool) -> Result<Program, UnbalancedBrackets> {
        // possible settings: https://docs.rs/cranelift-codegen/latest/src/cranelift_codegen/opt/rustwide/target/x86_64-unknown-linux-gnu/debug/build/cranelift-codegen-b5deaeb0cd154533/out/settings.rs.html#490-664
        let mut builder = settings::builder();
        builder.set("opt_level", "speed").unwrap();
        // issue: https://github.com/bytecodealliance/wasmtime/issues/1148
        builder.set("preserve_frame_pointers", "false").unwrap();
        // builder.set("use_egraphs", "true").unwrap();

        let flags = settings::Flags::new(builder);

        let isa = match isa::lookup(Triple::host()) {
            Err(_) => panic!("x86_64 ISA is not avaliable"),
            Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
        };

        let pointer_type = isa.pointer_type();

        // get memory address parameter, and return pointer to io::Error
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(pointer_type));
        sig.returns.push(AbiParam::new(pointer_type));

        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);

        let mut func_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

        let block = builder.create_block();
        builder.seal_block(block);

        builder.append_block_params_for_function_params(block);
        builder.switch_to_block(block);

        let pointer = Variable::new(0);
        builder.declare_var(pointer, pointer_type);

        let zero = builder.ins().iconst(pointer_type, 0);
        builder.def_var(pointer, zero);

        let memory_address = builder.block_params(block)[0];

        let mem_flags = MemFlags::new(); //.with_notrap().with_heap();

        let mut write_sig = Signature::new(CallConv::SystemV);
        write_sig.params.push(AbiParam::new(I8));
        write_sig.returns.push(AbiParam::new(pointer_type));
        let write_sig = builder.import_signature(write_sig);

        let write_address = write as *const () as i64;
        let write_address = builder.ins().iconst(pointer_type, write_address);

        let mut read_sig = Signature::new(CallConv::SystemV);
        read_sig.params.push(AbiParam::new(pointer_type));
        read_sig.returns.push(AbiParam::new(pointer_type));
        let read_sig = builder.import_signature(read_sig);

        let read_address = read as *const () as i64;
        let read_address = builder.ins().iconst(pointer_type, read_address);

        let mut stack = Vec::new();

        for (i, b) in source.iter().enumerate() {
            match b {
                b'+' => {
                    let pointer_value = builder.use_var(pointer);
                    let cell_address = builder.ins().iadd(memory_address, pointer_value);
                    let cell_value = builder.ins().load(I8, mem_flags, cell_address, 0);
                    let cell_value = builder.ins().iadd_imm(cell_value, 1);
                    builder.ins().store(mem_flags, cell_value, cell_address, 0);
                }
                b'-' => {
                    let pointer_value = builder.use_var(pointer);
                    let cell_address = builder.ins().iadd(memory_address, pointer_value);
                    let cell_value = builder.ins().load(I8, mem_flags, cell_address, 0);
                    let cell_value = builder.ins().iadd_imm(cell_value, -1);
                    builder.ins().store(mem_flags, cell_value, cell_address, 0);
                }
                b'.' => {
                    let pointer_value = builder.use_var(pointer);
                    let cell_address = builder.ins().iadd(memory_address, pointer_value);
                    let cell_value = builder.ins().load(I8, mem_flags, cell_address, 0);

                    builder
                        .ins()
                        .call_indirect(write_sig, write_address, &[cell_value]);
                }
                b',' => {
                    let pointer_value = builder.use_var(pointer);
                    let cell_address = builder.ins().iadd(memory_address, pointer_value);

                    builder
                        .ins()
                        .call_indirect(read_sig, read_address, &[cell_address]);
                }
                b'<' => {
                    let pointer_value = builder.use_var(pointer);
                    let pointer_minus = builder.ins().iadd_imm(pointer_value, -1);

                    let len = builder.ins().iconst(pointer_type, 30_000 - 1);
                    let pointer_value = builder.ins().select(pointer_value, pointer_minus, len);

                    builder.def_var(pointer, pointer_value);
                }
                b'>' => {
                    let pointer_value = builder.use_var(pointer);
                    let pointer_plus = builder.ins().iadd_imm(pointer_value, 1);

                    let cmp = builder.ins().icmp_imm(IntCC::Equal, pointer_value, 30_000);
                    let pointer_value = builder.ins().select(cmp, zero, pointer_plus);

                    builder.def_var(pointer, pointer_value);
                }
                b'[' => {
                    let inner_block = builder.create_block();
                    let after_block = builder.create_block();

                    let pointer_value = builder.use_var(pointer);
                    let cell_address = builder.ins().iadd(memory_address, pointer_value);
                    let cell_value = builder.ins().load(I8, mem_flags, cell_address, 0);

                    builder.ins().brz(cell_value, after_block, &[]);
                    builder.ins().jump(inner_block, &[]);

                    builder.switch_to_block(inner_block);

                    stack.push((inner_block, after_block));
                }
                b']' => {
                    let (inner_block, after_block) = match stack.pop() {
                        Some(x) => x,
                        None => return Err(UnbalancedBrackets(']', i)),
                    };

                    let pointer_value = builder.use_var(pointer);
                    let cell_address = builder.ins().iadd(memory_address, pointer_value);
                    let cell_value = builder.ins().load(I8, mem_flags, cell_address, 0);

                    builder.ins().brnz(cell_value, inner_block, &[]);
                    builder.ins().jump(after_block, &[]);

                    builder.seal_block(inner_block);
                    builder.seal_block(after_block);

                    builder.switch_to_block(after_block);
                }
                _ => continue,
            }
        }

        if !stack.is_empty() {
            return Err(UnbalancedBrackets(']', source.len()));
        }

        builder.ins().return_(&[zero]);

        builder.finalize();

        let res = verify_function(&func, &*isa);

        if clir {
            println!("{}", func.display());
        }

        if let Err(errors) = res {
            panic!("{}", errors);
        }

        let mut ctx = Context::for_function(func);
        let code = match ctx.compile(&*isa) {
            Ok(x) => x,
            Err(err) => {
                eprintln!("error compiling: {:?}", err);
                if clir {
                    println!("{}", ctx.func.display());
                }
                std::process::exit(4);
            }
        };

        let code = code.code_buffer().to_vec();

        if clir {
            println!("{}", ctx.func.display());
        }

        Ok(Program {
            code,
            memory: [0; 30_000],
        })
    }

    fn run(&mut self) -> std::io::Result<()> {
        let mut buffer = memmap2::MmapOptions::new()
            .len(self.code.len())
            .map_anon()
            .unwrap();

        buffer.copy_from_slice(self.code.as_slice());

        let buffer = buffer.make_exec().unwrap();

        unsafe {
            let code_fn: unsafe extern "sysv64" fn(*mut u8) -> *mut std::io::Error =
                std::mem::transmute(buffer.as_ptr());

            let error = code_fn(self.memory.as_mut_ptr());

            if !error.is_null() {
                return Err(*Box::from_raw(error));
            }
        }

        Ok(())
    }
}

extern "sysv64" fn write(value: u8) -> *mut std::io::Error {
    // Writing a non-UTF-8 byte sequence on Windows error out.
    if cfg!(target_os = "windows") && value >= 128 {
        return std::ptr::null_mut();
    }

    let mut stdout = std::io::stdout().lock();

    let result = stdout.write_all(&[value]).and_then(|_| stdout.flush());

    match result {
        Err(err) => Box::into_raw(Box::new(err)),
        _ => std::ptr::null_mut(),
    }
}

unsafe extern "sysv64" fn read(buf: *mut u8) -> *mut std::io::Error {
    let mut stdin = std::io::stdin().lock();
    loop {
        let mut value = 0;
        let err = stdin.read_exact(std::slice::from_mut(&mut value));

        if let Err(err) = err {
            if err.kind() != std::io::ErrorKind::UnexpectedEof {
                return Box::into_raw(Box::new(err));
            }
            value = 0;
        }

        // ignore CR from Window's CRLF
        if cfg!(target_os = "windows") && value == b'\r' {
            continue;
        }

        *buf = value;

        return std::ptr::null_mut();
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args();

    let mut dump = None;
    let mut source = None;
    let mut clir = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-d" | "--dump" => {
                dump = args.next();
                assert!(dump.is_some());
            }
            "--CLIR" => {
                clir = true;
            }
            _ => source = Some(arg),
        }
    }

    let source = match source {
        Some(x) => x,
        None => {
            eprintln!("expected a file path as argument");
            return ExitCode::from(1);
        }
    };

    let source = match std::fs::read(&source) {
        Ok(x) => x,
        Err(err) => {
            eprintln!("Error reading '{}': {}", source, err);
            return ExitCode::from(2);
        }
    };

    let mut program = match Program::new(&source, clir) {
        Ok(x) => x,
        Err(UnbalancedBrackets(c, address)) => {
            eprintln!(
                "Error parsing file: didn't found pair for `{}` at byte index {}",
                c, address
            );
            return ExitCode::from(3);
        }
    };

    if let Some(dump) = &dump {
        std::fs::write(dump, program.code.as_slice()).unwrap();
    }

    if dump.is_some() || clir {
        return ExitCode::from(0);
    }

    if let Err(err) = program.run() {
        eprintln!("IO error: {}", err);
    }

    ExitCode::from(0)
}
