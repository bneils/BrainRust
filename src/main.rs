use std::str;
use std::io;
use std::io::{Read, Write};
use regex::Regex;

use std::env;
use std::fs::File;
//use std::io::prelude::*;
use std::path::Path;

fn compile_jump_table(src: &Vec<Opcode>) -> Result<Vec<usize>, String> {
    let mut table = Vec::with_capacity(src.len());
    let mut stack = Vec::new();
    
    for _ in 0..src.len() {
        table.push(0);
    }

    for i in 0..src.len() {
        match src[i] {
            Opcode::BeginLoop => stack.push(i),
            Opcode::EndLoop => {
                match stack.pop() {
                    Some(left_bracket) => {
                        table[left_bracket] = i;
                        table[i] = left_bracket;
                    },
                    None => return Err(format!("Mismatched ']' at {}", i)),
                }
            },
            _ => {},
        }
    }

    if stack.len() == 0 {
        Ok(table)
    } else {
        Err(format!("{} too many '['", stack.len()))
    }
}

pub fn brainf(src: &str) {
    // A wrapper around the brainf interpreter, passing stdout to it.
    brainf_output(src, &mut io::stdout());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Opcode {
    /*
    These opcodes may carry a number of times to be executed.
    Some loops may require a lot of arithmetic that can be simplified.
    Add/Sub and Left/Right opcodes use a runlength encoding that groups the largest
    group of adjacent like instructions, and finds their net effect.
    There is no need to optimize empty loops as they will never be run anyways.
    */
    Add(u8), // the cells are u8 anyways, so no need to store more
    Sub(u8),
    ShiftLeft(usize),
    ShiftRight(usize),
    Print,
    Input,
    BeginLoop,
    EndLoop,
}

fn compile_opcodes(src: &str) -> Vec<Opcode> {
    let mut opcodes = Vec::new();
    let mut src = String::from(src);
    src.retain(|c| "+-<>[].,".contains(c));

    for m in Regex::new(r"[-+]+|[<>]+|\.|,|\[|\]").unwrap().find_iter(src.as_str()) {
        let match_str = m.as_str().as_bytes();
        match match_str[0] {
            b'+' | b'-' => {
                let num_minus = match_str.iter().filter(|b| **b == b'-').count();
                let num_plus = match_str.len() - num_minus;
                // num_plus = len - num_minus
                // net = num_plus - num_minus = (len - num_minus) - num_minus = len - 2num_minus
                if num_plus != num_minus {
                    opcodes.push(
                        if num_plus > num_minus { 
                            Opcode::Add((num_plus - num_minus) as u8)
                        } else { 
                            Opcode::Sub((num_minus - num_plus) as u8)
                        }
                    );
                }
            },
            b'<' | b'>' => {
                let num_left = match_str.iter().filter(|b| **b == b'<').count();
                let num_right = match_str.len() - num_left;
                if num_right != num_left {
                    opcodes.push(
                        if num_right > num_left { 
                            Opcode::ShiftRight(num_right - num_left)
                        } else { 
                            Opcode::ShiftLeft(num_left - num_right)
                        }
                    );
                }
            },
            b'.' => opcodes.push(Opcode::Print),
            b',' => opcodes.push(Opcode::Input),
            b'[' => opcodes.push(Opcode::BeginLoop),
            b']' => opcodes.push(Opcode::EndLoop),
            _ => {},
        }
    }
    opcodes
}

fn brainf_output(src: &str, stdout: &mut dyn Write) {
    let mut tape: Vec<u8> = vec![0];    // The number tape. You can move left or right on this.
    let mut tape_pos: usize = 0;
    
    let opcodes = compile_opcodes(src);
    let mut program_counter: usize = 0;

    let table = compile_jump_table(&opcodes).expect("Mismatched brackets");
    
    while program_counter < opcodes.len() {
        match opcodes[program_counter] {
            Opcode::Add(v) => tape[tape_pos] = tape[tape_pos].wrapping_add(v),
            Opcode::Sub(v) => tape[tape_pos] = tape[tape_pos].wrapping_sub(v),
            Opcode::ShiftLeft(shift) => {
                if tape_pos < shift {
                    for _ in 0..shift - tape_pos {
                        tape.insert(0, 0);
                    }
                    tape_pos = 0;
                } else {
                    tape_pos -= shift;
                }
            },
            Opcode::ShiftRight(shift) => {
                tape_pos += shift;
                while tape_pos >= tape.len() {
                    tape.push(0);
                }
            },
            Opcode::Print => {
                write!(stdout, "{}", str::from_utf8(&[tape[tape_pos]]).ok().unwrap()).expect("Could not write");
                stdout.flush().unwrap();
            },
            Opcode::Input => {
                tape[tape_pos] = io::stdin().bytes().next().expect("no byte read").unwrap();
            },
            Opcode::BeginLoop => {
                if tape[tape_pos] == 0 {
                    program_counter = table[program_counter];
                }
            },
            Opcode::EndLoop => {
                if tape[tape_pos] != 0 {
                    program_counter = table[program_counter];
                }
            },
        }
        program_counter += 1;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 0 {
        panic!("At least one brainfuck file path must be specified.");
    }

    for arg in &args[1..] {
        let path = Path::new(&arg);
        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", path.display(), why),
            Ok(file) => file,
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Err(why) => panic!("couldn't read {}: {}", path.display(), why),
            Ok(_) => {},
        }

        if args.len() > 2 {
            println!("Running {}", path.display());
        }
        brainf(&contents);
    }
}

#[cfg(test)]
mod tests {
    use crate::{compile_jump_table, brainf_output, brainf, compile_opcodes, Opcode};

    #[test]
    fn brackets_table_works() {
        let table = compile_jump_table(&compile_opcodes("++[-][]")).ok().unwrap();
        let expected = [0, 3, 0, 1, 5, 4];
        assert_eq!(table, expected);

        let table = compile_jump_table(&compile_opcodes("[]")).ok().unwrap();
        let expected = [1, 0];
        assert_eq!(table, expected);
    }

    #[test]
    fn prints_hello_world() {
        let src = ">+++IGNORED BY INTERPRETER+++++[<+++++++++>-]<.>++++[<+++++REDUNDANT COMMENT!!!++>-]<+.+++++++..+++.>>++++++[<+++++++>-]<+
        +.------------.>++++++[<+++++++++>-]<+.<.+++.------.-IGNORED BY INTERPRETER-------.>>>++++[<++++++++>-
        ]<+.";

        let mut output = Vec::new();
        brainf_output(src, &mut output);
        assert_eq!(output, b"Hello, World!");
    }

    #[test]
    fn skips_loop_at_beginning() {
        let src = "[+.]";
        let mut output = Vec::new();
        brainf_output(src, &mut output);
        assert_eq!(output.len(), 0);
    }

    #[test]
    fn opcodes_get_simplified() {
        assert_eq!(
            compile_opcodes(">++-[-],.<"),
            vec![
                Opcode::ShiftRight(1),
                Opcode::Add(1),
                Opcode::BeginLoop,
                Opcode::Sub(1),
                Opcode::EndLoop,
                Opcode::Input,
                Opcode::Print,
                Opcode::ShiftLeft(1),
            ]
        );

        assert_eq!(
            compile_opcodes(""),
            vec![]
        );

        assert_eq!(
            compile_opcodes("+-[[---]],..."),
            vec![
                Opcode::BeginLoop,
                Opcode::BeginLoop,
                Opcode::Sub(3),
                Opcode::EndLoop,
                Opcode::EndLoop,
                Opcode::Input,
                Opcode::Print,
                Opcode::Print,
                Opcode::Print,
            ]
        );
    }

    #[test]
    fn halting_loop_behavior() {
        brainf("++[-]"); // does not halt!
        brainf("--[+]"); // <--
        brainf(">++[-<->]<[+]");
    }
}