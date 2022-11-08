use std::io::{Read, Write};
use std::process::ExitCode;

use dynasmrt::mmap::MutableBuffer;
use dynasmrt::{dynasm, x64::X64Relocation, DynasmApi, DynasmLabelApi, VecAssembler};

#[derive(PartialEq, Eq, Clone, Copy)]
enum Instruction {
    Add(i8),
    Move(i32),
    Input,
    Output,
    JumpRight,
    JumpLeft,
    Clear,
    AddTo(i32),
    MoveUntil(i32),
}

struct UnbalancedBrackets(char, usize);

struct Program {
    code: Vec<u8>,
    memory: [u8; 30_000],
}
impl Program {
    fn new(source: &[u8]) -> Result<Program, UnbalancedBrackets> {
        let mut code: VecAssembler<X64Relocation> = VecAssembler::new(0);

        let mut instructions = Vec::new();

        for b in source {
            let instr = match b {
                b'+' | b'-' => {
                    let inc = if *b == b'+' { 1 } else { -1 };
                    if let Some(Instruction::Add(value)) = instructions.last_mut() {
                        *value = value.wrapping_add(inc);
                        continue;
                    }
                    Instruction::Add(inc)
                }
                b'.' => Instruction::Output,
                b',' => Instruction::Input,
                b'>' | b'<' => {
                    let inc = if *b == b'>' { 1 } else { -1 };
                    if let Some(Instruction::Move(value)) = instructions.last_mut() {
                        *value += inc;
                        continue;
                    }
                    Instruction::Move(inc)
                }
                b'[' => Instruction::JumpRight,
                b']' => {
                    use Instruction::*;
                    match instructions.as_slice() {
                        // could enter a infinite loop if n is even.
                        [.., JumpRight, Add(n)] if n % 2 == 1 => {
                            let len = instructions.len();
                            instructions.drain(len - 2..);
                            Instruction::Clear
                        }
                        &[.., JumpRight, Add(-1), Move(x), Add(1), Move(y)] if x == -y => {
                            let len = instructions.len();
                            instructions.drain(len - 5..);
                            Instruction::AddTo(x)
                        }
                        &[.., JumpRight, Move(n)] => {
                            let len = instructions.len();
                            instructions.drain(len - 2..);
                            Instruction::MoveUntil(n)
                        }
                        _ => Instruction::JumpLeft,
                    }
                }
                _ => continue,
            };
            instructions.push(instr);
        }

        // r12 will be the adress of `memory`
        // r13 will be the value of `pointer`
        // r12 is got from argument 1 in `rdi`
        // r13 is set to 0
        dynasm! { code
            ; .arch x64
            ; push rbp
            ; mov rbp, rsp
            ; push r12
            ; push r13
            ; mov r12, rdi
            ; xor r13, r13
        };

        let mut bracket_stack = Vec::new();

        for instr in instructions.into_iter() {
            match instr {
                Instruction::Add(n) => dynasm! { code
                    ; .arch x64
                    ; add BYTE [r12 + r13], BYTE n as i8
                },
                Instruction::Move(n) => {
                    if n > 0 {
                        dynasm! { code
                            ; lea eax, [r13 + n]
                            ; add r13, -(30000 - n)
                            ; cmp eax, 30000
                            ; cmovl	r13d, eax
                        }
                    } else {
                        dynasm! { code
                            ; lea eax, [r13 + n]
                            ; add r13d, 30000 + n
                            ; test eax, eax
                            ; cmovns r13d, eax
                        }
                    }
                }
                Instruction::Input => {
                    dynasm! { code
                        ; .arch x64
                        ; mov rax, QWORD read as *const () as i64
                        ; lea rdi, [r12 + r13] // cell address
                        ; call rax
                        ; cmp rax, 0
                        ; jne ->exit
                    }
                }
                Instruction::Output => {
                    dynasm! { code
                        ; .arch x64
                        ; mov rax, QWORD write as *const () as i64
                        ; mov rdi, [r12 + r13] // cell value
                        ; call rax
                        ; cmp rax, 0
                        ; jne ->exit
                    }
                }
                Instruction::JumpRight => {
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
                Instruction::JumpLeft => {
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
                Instruction::Clear => dynasm! { code
                    ; .arch x64
                    ; mov BYTE [r12 + r13], 0
                },
                Instruction::AddTo(n) => dynasm! { code
                    ; .arch x64
                    // rax = cell to add to
                    ;;
                    if n > 0 {
                        dynasm! { code
                            ; lea ecx, [r13 + n]
                            ; lea eax, [r13 + n - 30000]
                            ; cmp ecx, 30000
                            ; cmovl eax, ecx
                        }
                    } else {
                        dynasm! { code
                            ; lea ecx, [r13 + n]
                            ; lea eax, [r13 + 30000 + n]
                            ; test ecx, ecx
                            ; cmovns eax, ecx
                        }
                    }
                    ; mov cl, [r12 + r13]
                    ; add BYTE [r12 + rax], cl
                    ; mov BYTE [r12 + r13], 0
                },
                Instruction::MoveUntil(n) => dynasm! { code
                    ; .arch x64

                    ; repeat:

                    // check if 0
                    ; cmp BYTE [r12 + r13], 0
                    ; je >exit

                    // Move n
                    ;;
                    if n > 0 {
                        dynasm! { code
                            ; lea eax, [r13 + n]
                            ; add r13, -(30000 - n)
                            ; cmp eax, 30000
                            ; cmovl r13d, eax
                        }
                    } else {
                        dynasm! { code
                            ; lea eax, [r13 + n]
                            ; add r13d, 30000 + n
                            ; test eax, eax
                            ; cmovns r13d, eax
                        }
                    }

                    ; jmp <repeat

                    ; exit:
                },
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
            ; pop rbp
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
