use std::{
    io::{Read, Write},
    process::ExitCode,
};

#[derive(PartialEq, Eq, Clone, Copy)]
enum Instruction {
    Add(u8),
    Move(isize),
    Input,
    Output,
    JumpRight(usize),
    JumpLeft(usize),
}

struct UnbalancedBrackets(char, usize);

#[derive(Default, Debug)]
#[cfg(feature = "profile")]
struct Profile {
    add: u64,
    mov: u64,
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
    fn new(source: &[u8]) -> Result<Program, UnbalancedBrackets> {
        let mut instructions = Vec::new();
        let mut bracket_stack = Vec::new();

        for b in source {
            let instr = match b {
                b'+' | b'-' => {
                    let inc = if *b == b'+' { 1 } else { 1u8.wrapping_neg() };
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
            #[cfg(feature = "profile")]
            profile: Profile::default(),
        })
    }

    fn run(&mut self) -> std::io::Result<()> {
        let mut stdout = std::io::stdout().lock();
        let mut stdin = std::io::stdin().lock();
        'program: loop {
            use Instruction::*;

            #[cfg(feature = "profile")]
            {
                match self.instructions[self.program_counter] {
                    Add(_) => self.profile.add += 1,
                    Output => self.profile.out += 1,
                    Input => self.profile.inp += 1,
                    Move(_) => self.profile.mov += 1,
                    JumpRight(_) => self.profile.jr += 1,
                    JumpLeft(_) => self.profile.jl += 1,
                }
            }

            match self.instructions[self.program_counter] {
                Add(n) => self.memory[self.pointer] = self.memory[self.pointer].wrapping_add(n),
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
                Move(n) => {
                    let len = self.memory.len() as isize;
                    let n = (len + n % len) as usize;
                    self.pointer = (self.pointer + n) % len as usize;
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

    #[cfg(feature = "profile")]
    {
        dbg!(program.profile);
    }

    ExitCode::from(0)
}
