use std::fs;
use std::collections::VecDeque;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Opcode {
    Add,
    Multiply,
    Input,
    Output,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equals,
    Return,
    Unknown,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Status {
    Running,
    Finished,
    Blocked,
    Killed,
}


#[derive(Clone, Debug)]
struct Intcode {
    status: Status,
    memory: Vec<i32>,
    program_counter: usize,
    inputs: VecDeque<i32>,
    outputs: VecDeque<i32>,
}

impl Intcode {
    fn new(program:Vec<i32>) -> Intcode {
        Intcode {
            status: Status::Running,
            memory: program,
            program_counter: 0,
            inputs: VecDeque::new(),
            outputs: VecDeque::new(),
        }
    }

    fn input(&mut self, value:i32) {
        self.inputs.push_back(value);
    }

    fn execute(&mut self) {
        while self.status == Status::Running {
            self.tick();
        }
    }

    fn tick(&mut self) {
        let instruction = self.memory[self.program_counter];
        let opcode = self.opcode(instruction);
        match opcode {
            Opcode::Add => {
                let a = self.get_parameter(1);
                let b = self.get_parameter(2);
                self.store_position(3, a + b);
                self.program_counter += 4;
            },
            Opcode::Multiply => {
                let a = self.get_parameter(1);
                let b = self.get_parameter(2);
                self.store_position(3, a * b);
                self.program_counter += 4;
            },
            Opcode::Input => {
                if let Some(input) = self.inputs.pop_back() {
                    self.store_position(1, input);
                    self.program_counter += 2;
                } else {
                    self.status = Status::Blocked;
                }
            },
            Opcode::Output => {
                let output = self.get_parameter(1);
                self.outputs.push_back(output);
                self.program_counter += 2;
            },
            Opcode::JumpIfTrue => {
                let condition = self.get_parameter(1);
                if condition != 0 {
                    let jump = self.get_parameter(2) as usize;
                    self.program_counter = jump;
                } else {
                    self.program_counter += 3;
                }
            },
            Opcode::JumpIfFalse => {
                let condition = self.get_parameter(1);
                if condition == 0 {
                    let jump = self.get_parameter(2) as usize;
                    self.program_counter = jump;
                } else {
                    self.program_counter += 3;
                }
            },
            Opcode::LessThan => {
                let a = self.get_parameter(1);
                let b = self.get_parameter(2);
                if a < b {
                    self.store_position(3, 1);
                } else {
                    self.store_position(3, 0);
                }
                self.program_counter += 4;
            },
            Opcode::Equals => {
                let a = self.get_parameter(1);
                let b = self.get_parameter(2);
                if a == b {
                    self.store_position(3, 1);
                } else {
                    self.store_position(3, 0);
                }
                self.program_counter += 4;
            },
            Opcode::Return => {
                self.status = Status::Finished;
            },
            Opcode::Unknown => {
                self.status = Status::Killed;
            },
        }
    }

    fn opcode(&mut self, instruction:i32) -> Opcode{
        match instruction % 100 {
            1 => Opcode::Add,
            2 => Opcode::Multiply,
            3 => Opcode::Input,
            4 => Opcode::Output,
            5 => Opcode::JumpIfTrue,
            6 => Opcode::JumpIfFalse,
            7 => Opcode::LessThan,
            8 => Opcode::Equals,
            99 => Opcode::Return,
            _ => Opcode::Unknown,
        }
    }

    fn get_parameter(&self, offset:usize) -> i32 {
        if self.is_immediate(offset) {
            self.get(offset)
        } else {
            self.get_position(offset)
        }
    }

    fn store_position(&mut self, offset:usize, value:i32) {
        let store_index = self.memory[self.program_counter + offset] as usize;
        self.memory[store_index] = value;
    }

    fn get(&self, offset:usize) -> i32 {
        self.memory[self.program_counter + offset]
    }

    fn get_position(&self, offset:usize) -> i32 {
        self.memory[self.get(offset) as usize]
    }

    fn is_immediate(&self, offset:usize) -> bool {
        (self.memory[self.program_counter] / (10i32.pow(offset as u32 + 1))) % 10 == 1
    }
}


fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let program: Vec<i32> = data.split(',')
                                    .map(|s| s.parse().unwrap())
                                    .collect();

    for &system_id in vec![1, 5].iter() {
        let mut prog = Intcode::new(program.clone());

        prog.input(system_id);
        prog.execute();

        println!("The program diagnostic code for system ID {:?} = {:?}", system_id, prog.outputs.pop_back().unwrap());
    }
}