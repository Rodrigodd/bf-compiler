use std::io::Write;
use std::process::ExitCode;

use dynasm::dynasm;

struct UnbalancedBrackets(char, usize);

trait DynasmApi {
    fn push_i8(&mut self, value: i8);
    fn push_i32(&mut self, value: i32);
}
impl DynasmApi for Vec<u8> {
    fn push_i8(&mut self, value: i8) {
        self.push(value as u8)
    }
    fn push_i32(&mut self, value: i32) {
        self.extend_from_slice(&value.to_le_bytes())
    }
}

struct Program {
    code: Vec<u8>,
    memory: [u8; 30_000],
}
impl Program {
    fn new(source: &[u8]) -> Result<Program, UnbalancedBrackets> {
        let mut code = Vec::new();

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
                    // note that the offset of 0 is a dummy value,
                    // it will be fixed in the pair `]`
                    dynasm! {code
                        ; .arch x64
                        ; cmp BYTE [r12+r13], 0
                        ; je i32::max_value()
                    };

                    // push to the stack the byte index of the next instruction.
                    bracket_stack.push(code.len() as u32);
                }
                b']' => {
                    let left = match bracket_stack.pop() {
                        Some(x) => x as usize,
                        None => return Err(UnbalancedBrackets(']', code.len())),
                    };

                    dynasm! {code
                        ; .arch x64
                        ; cmp BYTE [r12 + r13], 0
                        ; jne i32::max_value()
                    };

                    // the byte index of the next instruction
                    let right = code.len();

                    let offset = right as i32 - left as i32;

                    // fix relative jumps offsets
                    code[left - 4..left].copy_from_slice(&offset.to_le_bytes());
                    code[right - 4..right].copy_from_slice(&(-offset).to_le_bytes());
                }
                _ => continue,
            }
        }

        if !bracket_stack.is_empty() {
            return Err(UnbalancedBrackets(']', code.len()));
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
            code,
            memory: [0; 30_000],
        })
    }

    fn run(&mut self) -> std::io::Result<()> {
        unsafe {
            let len = self.code.len();
            let mem = libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            );

            if mem == libc::MAP_FAILED {
                panic!("mmap failed");
            }

            // SAFETY: mem is zero initalized by the mmap.
            std::slice::from_raw_parts_mut(mem as *mut u8, len).copy_from_slice(&self.code);

            // mem.as_ptr() is page aligned, because it is get from mmap.
            let result = libc::mprotect(mem, len, libc::PROT_READ | libc::PROT_EXEC);

            if result == -1 {
                panic!("mprotect failed");
            }

            let code_fn: unsafe extern "sysv64" fn(*mut u8) = std::mem::transmute(mem);

            code_fn(self.memory.as_mut_ptr());

            let result = libc::munmap(mem, len);

            if result == -1 {
                panic!("munmap failed");
            }
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
