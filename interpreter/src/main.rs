use std::io::Read;

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

struct Program {
    program_counter: usize,
    pointer: usize,
    instructions: Vec<Instruction>,
    memory: [u8; 30_000],
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
        }
    }

    fn run(&mut self) -> std::io::Result<()> {
        'program: loop {
            use Instruction::*;
            match self.instructions[self.program_counter] {
                Increase => self.memory[self.pointer] = self.memory[self.pointer].wrapping_add(1),
                Decrease => self.memory[self.pointer] = self.memory[self.pointer].wrapping_sub(1),
                Output => print!("{}", self.memory[self.pointer] as char),
                Input => {
                    std::io::stdin().read_exact(&mut self.memory[self.pointer..self.pointer + 1])?
                }
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

fn main() -> std::io::Result<()> {
    let file_name = std::env::args().nth(1).unwrap();
    let source = std::fs::read(&file_name)?;

    Program::new(&source).run()
}
