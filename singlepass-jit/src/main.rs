use std::io::{Read, Write};
use std::process::ExitCode;

use dynasmrt::mmap::MutableBuffer;
use dynasmrt::{dynasm, x64::X64Relocation, DynasmApi, DynasmLabelApi, VecAssembler};

struct UnbalancedBrackets(char, usize);

struct Program {
    code: Vec<u8>,
    memory: [u8; 30_000],
}
impl Program {
    fn new(source: &[u8]) -> Result<Program, UnbalancedBrackets> {
        let mut code: VecAssembler<X64Relocation> = VecAssembler::new(0);

        // r12 will be the adress of `memory`
        // r13 will be the value of `pointer`
        // r12 is got from argument 1 in `rdi`
        // r13 is set to 0
        dynasm! { code
            ; .arch x64
            ; push r12
            ; push r13
            ; mov r12, rdi
            ; xor r13, r13
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
                b'.' => {
                    dynasm! { code
                        ; .arch x64
                        ; mov rax, QWORD write as *const () as i64
                        ; mov rdi, [r12 + r13] // cell value
                        ; call rax
                        ; cmp rax, 0
                        ; jne ->exit
                    }
                }
                b',' => {
                    dynasm! { code
                        ; .arch x64
                        ; mov rax, QWORD read as *const () as i64
                        ; lea rdi, [r12 + r13] // cell address
                        ; call rax
                        ; cmp rax, 0
                        ; jne ->exit
                    }
                }
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

        // when we push to the stack, we need to remeber
        // to pop them in the opossite order.
        dynasm! { code
            ; .arch x64
            ; xor rax, rax
            ; ->exit:
            ; pop r13
            ; pop r12
            ; ret
        }

        Ok(Program {
            code: code.finalize().unwrap(),
            memory: [0; 30_000],
        })
    }

    fn run(&mut self) -> std::io::Result<()> {
        let mut buffer = MutableBuffer::new(self.code.len()).unwrap();
        buffer.set_len(self.code.len());

        buffer.copy_from_slice(&self.code);

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
        Err(err) => Box::leak(Box::new(err)) as *mut _,
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
                return Box::leak(Box::new(err));
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
    if args.len() != 2 {
        eprintln!("expected a single file path as argument");
        return ExitCode::from(1);
    }

    let file_name = args.nth(1).unwrap();
    let source = match std::fs::read(&file_name) {
        Ok(x) => x,
        Err(err) => {
            eprintln!("Error reading '{}': {}", file_name, err);
            return ExitCode::from(2);
        }
    };

    let mut program = match Program::new(&source) {
        Ok(x) => x,
        Err(UnbalancedBrackets(c, address)) => {
            eprintln!(
                "Error parsing file: didn't found pair for `{}` at instruction index {}",
                c, address
            );
            return ExitCode::from(3);
        }
    };

    if let Err(err) = program.run() {
        eprintln!("IO error: {}", err);
    }

    ExitCode::from(0)
}
