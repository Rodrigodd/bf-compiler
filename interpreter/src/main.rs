fn main() -> std::io::Result<()> {
    let file_name = std::env::args().nth(1).unwrap();
    let source = std::fs::read(&file_name)?;

    let mut pointer = 0;
    let mut program_counter = 0;
    let mut memory = [0u8; 30_000];
    'program: loop {
        match source[program_counter] {
            b'+' => memory[pointer] = memory[pointer].wrapping_add(1),
            b'-' => memory[pointer] = memory[pointer].wrapping_sub(1),
            b'.' => print!("{}", memory[pointer] as char),
            b',' => {
                use std::io::Read;
                std::io::stdin().read_exact(&mut memory[pointer..pointer + 1])?
            }
            b'>' => pointer = (pointer + 1) % memory.len(),
            b'<' => pointer = (pointer + memory.len() - 1) % memory.len(),
            b'[' => {
                if memory[pointer] == 0 {
                    let mut deep = 1;
                    loop {
                        if program_counter + 1 == source.len() {
                            break 'program;
                        }
                        program_counter += 1;
                        if source[program_counter] == b'[' {
                            deep += 1;
                        }
                        if source[program_counter] == b']' {
                            deep -= 1;
                        }
                        if deep == 0 {
                            break;
                        }
                    }
                }
            }
            b']' => {
                if memory[pointer] != 0 {
                    let mut deep = 1;
                    loop {
                        if program_counter == 0 {
                            break 'program;
                        }
                        program_counter -= 1;
                        if source[program_counter] == b']' {
                            deep += 1;
                        }
                        if source[program_counter] == b'[' {
                            deep -= 1;
                        }
                        if deep == 0 {
                            break;
                        }
                    }
                }
            }
            _ => {} // do nothing, its only a comment
        }
        program_counter += 1;

        if source.len() == program_counter {
            break 'program;
        }
    }
    Ok(())
}
