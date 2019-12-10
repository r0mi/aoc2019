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
                if let Some(input) = self.inputs.pop_front() {
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

    fn finished(&self) -> bool {
        self.status == Status::Finished
    }

    fn blocked(&self) -> bool {
        self.status == Status::Blocked
    }

    fn r#continue(&mut self) {
        if self.blocked() && self.inputs.len() > 0 {
            self.status = Status::Running;
        }
        self.execute();
    }
}

fn permute(list:&mut Vec<i32>, start_idx:usize, end_idx:usize, permutes:&mut Vec<Vec<i32>>) {
    if start_idx == end_idx {
        permutes.push(list.clone());
    } else {
        for i in start_idx..end_idx {
            let mut tmp = list[start_idx];
            list[start_idx] = list[i];
            list[i] = tmp;
            permute(list, start_idx + 1, end_idx, permutes);
            tmp = list[start_idx];
            list[start_idx] = list[i];
            list[i] = tmp;
        }
    }
}

fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let program: Vec<i32> = data.split(',')
                                    .map(|s| s.parse().unwrap())
                                    .collect();

    let mut phase_setting_permutations:Vec<Vec<i32>> = vec![];
    permute(&mut (0..5).collect(), 0, 5, &mut phase_setting_permutations);
    let mut max_thruster_signal = 0;

    for setting in phase_setting_permutations {
        let mut last_output:i32 = 0;

        for phase in setting {
            let mut amp = Intcode::new(program.clone());
            amp.input(phase);
            amp.input(last_output);
            amp.execute();
            last_output = amp.outputs.pop_back().expect("Nan");
        }
        if last_output > max_thruster_signal {
            max_thruster_signal = last_output;
        }
    }

    println!("Part1: The highest signal that can be sent to the thrusters is {:?}", max_thruster_signal);

    let mut phase_setting_permutations:Vec<Vec<i32>> = vec![];
    permute(&mut (5..10).collect(), 0, 5, &mut phase_setting_permutations);
    max_thruster_signal = 0;

    for setting in phase_setting_permutations {
        let mut last_output:i32 = 0;

        let mut amps:Vec<Intcode> = vec![];

        for x in 0..5 {
            let mut amp = Intcode::new(program.clone());
            amp.input(setting[x]);
            amps.push(amp);
        }

        let mut finished = false;

        while !finished {
            for amp in &mut amps {
                amp.input(last_output);
                if amp.blocked() {
                    amp.r#continue();
                } else {
                    amp.execute();
                }
                match amp.outputs.pop_back() {
                    Some(value) => last_output = value,
                    None => panic!("{:?} no output", setting),
                }
                finished = amp.finished();
            }
        }

        if last_output > max_thruster_signal {
            max_thruster_signal = last_output;
        }
    }

    println!("Part2: The highest signal that can be sent to the thrusters is {:?}", max_thruster_signal);
}