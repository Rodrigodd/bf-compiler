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

fn main() -> std::io::Result<()> {
    let file_name = std::env::args().nth(1).unwrap();
    let source = std::fs::read(&file_name)?;

    let source: Vec<Instruction> = source
        .into_iter()
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

    let mut pointer = 0;
    let mut program_counter = 0;
    let mut memory = [0u8; 30_000];
    'program: loop {
        use Instruction::*;
        match source[program_counter] {
            Increase => memory[pointer] = memory[pointer].wrapping_add(1),
            Decrease => memory[pointer] = memory[pointer].wrapping_sub(1),
            Output => print!("{}", memory[pointer] as char),
            Input => std::io::stdin().read_exact(&mut memory[pointer..pointer + 1])?,
            MoveRight => pointer = (pointer + 1) % memory.len(),
            MoveLeft => pointer = (pointer + memory.len() - 1) % memory.len(),
            JumpRight => {
                if memory[pointer] == 0 {
                    let mut deep = 1;
                    loop {
                        if program_counter + 1 == source.len() {
                            break 'program;
                        }
                        program_counter += 1;
                        if source[program_counter] == JumpRight {
                            deep += 1;
                        }
                        if source[program_counter] == JumpLeft {
                            deep -= 1;
                        }
                        if deep == 0 {
                            break;
                        }
                    }
                }
            }
            JumpLeft => {
                if memory[pointer] != 0 {
                    let mut deep = 1;
                    loop {
                        if program_counter == 0 {
                            break 'program;
                        }
                        program_counter -= 1;
                        if source[program_counter] == JumpLeft {
                            deep += 1;
                        }
                        if source[program_counter] == JumpRight {
                            deep -= 1;
                        }
                        if deep == 0 {
                            break;
                        }
                    }
                }
            }
        }
        program_counter += 1;

        if source.len() == program_counter {
            break 'program;
        }
    }
    Ok(())
}
