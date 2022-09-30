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
    JumpRight,
    JumpLeft,
}

#[derive(Default, Debug)]
#[cfg(feature = "profile")]
struct Profile {
    inc: u64,
    dec: u64,
    movr: u64,
    movl: u64,
    jr: u64,
    jl: u64,
    inp: u64,
    out: u64,
}

struct Program {
    program_counter: usize,
    pointer: usize,
    instructions: Vec<Instruction>,
    memory: [u8; 30_000],
    #[cfg(feature = "profile")]
    profile: Profile,
}
impl Program {
    fn new(source: &[u8]) -> Program {
        let instructions: Vec<Instruction> = source
            .iter()
            .filter_map(|b| match b {
                b'+' => Some(Instruction::Increase),
                b'-' => Some(Instruction::Decrease),
                b'.' => Some(Instruction::Output),
                b',' => Some(Instruction::Input),
                b'>' => Some(Instruction::MoveRight),
                b'<' => Some(Instruction::MoveLeft),
                b'[' => Some(Instruction::JumpRight),
                b']' => Some(Instruction::JumpLeft),
                _ => None,
            })
            .collect();

        Program {
            program_counter: 0,
            pointer: 0,
            instructions,
            memory: [0; 30_000],
            #[cfg(feature = "profile")]
            profile: Profile::default(),
        }
    }

    fn run(&mut self) -> std::io::Result<()> {
        let mut stdout = std::io::stdout().lock();
        let mut stdin = std::io::stdin().lock();
        'program: loop {
            use Instruction::*;

            #[cfg(feature = "profile")]
            {
                match self.instructions[self.program_counter] {
                    Increase => self.profile.inc += 1,
                    Decrease => self.profile.dec += 1,
                    Output => self.profile.out += 1,
                    Input => self.profile.inp += 1,
                    MoveRight => self.profile.movr += 1,
                    MoveLeft => self.profile.movl += 1,
                    JumpRight => self.profile.jr += 1,
                    JumpLeft => self.profile.jl += 1,
                }
            }

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
                JumpRight => {
                    if self.memory[self.pointer] == 0 {
                        let mut deep = 1;
                        loop {
                            if self.program_counter + 1 == self.instructions.len() {
                                break 'program;
                            }
                            self.program_counter += 1;
                            if self.instructions[self.program_counter] == JumpRight {
                                deep += 1;
                            }
                            if self.instructions[self.program_counter] == JumpLeft {
                                deep -= 1;
                            }
                            if deep == 0 {
                                break;
                            }
                        }
                    }
                }
                JumpLeft => {
                    if self.memory[self.pointer] != 0 {
                        let mut deep = 1;
                        loop {
                            if self.program_counter == 0 {
                                break 'program;
                            }
                            self.program_counter -= 1;
                            if self.instructions[self.program_counter] == JumpLeft {
                                deep += 1;
                            }
                            if self.instructions[self.program_counter] == JumpRight {
                                deep -= 1;
                            }
                            if deep == 0 {
                                break;
                            }
                        }
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

    let mut program = Program::new(&source);

    if let Err(err) = program.run() {
        eprintln!("IO error: {}", err);
    }

    #[cfg(feature = "profile")]
    {
        dbg!(program.profile);
    }

    ExitCode::from(0)
}
