use std::{
    io::{Read, Write},
    process::ExitCode,
};

#[derive(PartialEq, Eq, Clone, Copy)]
enum Instruction {
    Increase,
    Decrease,
    MoveRight,
    MoveLeft,
    Input,
    Output,
    JumpRight(usize),
    JumpLeft(usize),
}

struct UnbalancedBrackets(char, usize);

struct Program {
    program_counter: usize,
    pointer: usize,
    instructions: Vec<Instruction>,
    memory: [u8; 30_000],
}
impl Program {
    fn new(source: &[u8]) -> Result<Program, UnbalancedBrackets> {
        let mut instructions = Vec::new();
        let mut bracket_stack = Vec::new();

        for b in source {
            let instr = match b {
                b'+' => Instruction::Increase,
                b'-' => Instruction::Decrease,
                b'.' => Instruction::Output,
                b',' => Instruction::Input,
                b'>' => Instruction::MoveRight,
                b'<' => Instruction::MoveLeft,
                b'[' => {
                    let curr_address = instructions.len();
                    bracket_stack.push(curr_address);
                    // will be fixup at the pair ']'.
                    Instruction::JumpRight(0)
                }
                b']' => {
                    let curr_address = instructions.len();
                    match bracket_stack.pop() {
                        Some(pair_address) => {
                            instructions[pair_address] = Instruction::JumpRight(curr_address);
                            Instruction::JumpLeft(pair_address)
                        }
                        None => return Err(UnbalancedBrackets(']', curr_address)),
                    }
                }
                _ => continue,
            };
            instructions.push(instr);
        }

        if let Some(unpaired_bracket) = bracket_stack.pop() {
            return Err(UnbalancedBrackets('[', unpaired_bracket));
        }

        Ok(Program {
            program_counter: 0,
            pointer: 0,
            instructions,
            memory: [0; 30_000],
        })
    }

    fn run(&mut self) -> std::io::Result<()> {
        let mut stdout = std::io::stdout().lock();
        let mut stdin = std::io::stdin().lock();
        'program: loop {
            use Instruction::*;
            match self.instructions[self.program_counter] {
                Increase => self.memory[self.pointer] = self.memory[self.pointer].wrapping_add(1),
                Decrease => self.memory[self.pointer] = self.memory[self.pointer].wrapping_sub(1),
                Output => {
                    let value = self.memory[self.pointer];
                    // Writing a non-UTF-8 byte sequence on Windows error out.
                    if !cfg!(target_os = "windows") || value < 128 {
                        stdout.write_all(&[value])?;
                        stdout.flush()?;
                    }
                }
                Input => loop {
                    let err = stdin.read_exact(&mut self.memory[self.pointer..self.pointer + 1]);
                    match err.as_ref().map_err(|e| e.kind()) {
                        Err(std::io::ErrorKind::UnexpectedEof) => {
                            self.memory[self.pointer] = 0;
                        }
                        _ => err?,
                    }
                    if cfg!(target_os = "windows") && self.memory[self.pointer] == b'\r' {
                        continue;
                    }
                    break;
                },
                MoveRight => self.pointer = (self.pointer + 1) % self.memory.len(),
                MoveLeft => {
                    self.pointer = (self.pointer + self.memory.len() - 1) % self.memory.len()
                }
                JumpRight(pair_address) => {
                    if self.memory[self.pointer] == 0 {
                        self.program_counter = pair_address;
                    }
                }
                JumpLeft(pair_address) => {
                    if self.memory[self.pointer] != 0 {
                        self.program_counter = pair_address;
                    }
                }
            }
            self.program_counter += 1;

            if self.instructions.len() == self.program_counter {
                break 'program;
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
