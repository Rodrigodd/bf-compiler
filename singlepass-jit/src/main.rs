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

        // when we push to the stack, we need to remeber
        // to pop them in the opossite order.
        dynasm! { code
            ; .arch x64
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
            let code_fn: unsafe extern "sysv64" fn(*mut u8) = std::mem::transmute(buffer.as_ptr());
            code_fn(self.memory.as_mut_ptr());
        }

        Ok(())
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
