use std::fs;
use std::env;
use std::usize;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::char;

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
    AdjustRelativeBase,
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

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum ParameterMode {
    Position,
    Immediate,
    Relative,
    Unknown,
}

#[derive(Clone, Debug)]
struct Intcode {
    status: Status,
    memory: HashMap<usize, i64>,
    program_counter: usize,
    relative_base: i64,
    inputs: VecDeque<i64>,
    outputs: VecDeque<i64>,
    live_feed: bool
}

impl Intcode {
    fn new(program:Vec<i64>) -> Intcode {
        let mut m:HashMap<usize, i64> = HashMap::new();
        for (i, &v) in program.iter().enumerate() {
            m.insert(i, v);
        }
        Intcode {
            status: Status::Running,
            memory: m,
            program_counter: 0,
            relative_base: 0,
            inputs: VecDeque::new(),
            outputs: VecDeque::new(),
            live_feed: false
        }
    }

    fn set_live_feed(&mut self, live:bool) {
        self.live_feed = live;
    }

    fn input(&mut self, value:i64) {
        self.inputs.push_back(value);
    }

    fn append_input(&mut self, values:&Vec<i64>) {
        self.inputs.append(&mut VecDeque::from(values.to_vec()));
    }

    fn execute(&mut self) {
        if self.inputs.len() > 0  && self.live_feed {
            self.print_input();
        }
        while self.status == Status::Running {
            self.tick();
        }
    }

    fn print_output(&mut self) {
        print!("{}", self.outputs.iter().map(|&c| c as u8 as char).collect::<String>());
        self.outputs.clear();
    }

    fn print_input(&self) {
        print!("{}", self.inputs.iter().map(|&c| c as u8 as char).collect::<String>());
    }

    fn tick(&mut self) {
        let instruction = self.memory[&self.program_counter];
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
                if self.live_feed && output == 10 {
                    self.print_output();
                }
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
            Opcode::AdjustRelativeBase => {
                let a = self.get_parameter(1);
                self.relative_base += a;
                self.program_counter += 2;
            },
            Opcode::Return => {
                self.status = Status::Finished;
            },
            Opcode::Unknown => {
                self.status = Status::Killed;
            },
        }
    }

    fn opcode(&mut self, instruction:i64) -> Opcode{
        match instruction % 100 {
            1 => Opcode::Add,
            2 => Opcode::Multiply,
            3 => Opcode::Input,
            4 => Opcode::Output,
            5 => Opcode::JumpIfTrue,
            6 => Opcode::JumpIfFalse,
            7 => Opcode::LessThan,
            8 => Opcode::Equals,
            9 => Opcode::AdjustRelativeBase,
            99 => Opcode::Return,
            _ => Opcode::Unknown,
        }
    }

    fn get_parameter(&mut self, offset:usize) -> i64 {
        match self.get_parameter_mode(offset) {
            ParameterMode::Position => self.get_position(offset),
            ParameterMode::Immediate => self.get(offset),
            ParameterMode::Relative => self.get_relative(offset),
            ParameterMode::Unknown => panic!("Unknown paramter mode"),
        }
    }

    fn store_position(&mut self, offset:usize, value:i64) {
        let store_index:usize;
        match self.get_parameter_mode(offset) {
            ParameterMode::Position => store_index = self.memory[&(self.program_counter + offset)] as usize,
            ParameterMode::Immediate => store_index = self.memory[&(self.program_counter + offset)] as usize,
            ParameterMode::Relative => store_index = (self.relative_base + self.memory[&(self.program_counter + offset)]) as usize,
            ParameterMode::Unknown => panic!("Unknown paramter mode"),
        }

        *self.memory.entry(store_index).or_insert(0) = value
    }

    fn get(&mut self, offset:usize) -> i64 {
        *self.memory.entry(self.program_counter + offset).or_insert(0)
    }

    fn get_position(&mut self, offset:usize) -> i64 {
        let v = self.get(offset) as usize;
        *self.memory.entry(v).or_insert(0)
    }

    fn get_relative(&mut self, offset:usize) -> i64 {
        let v = self.get(offset);
        *self.memory.entry((self.relative_base + v) as usize).or_insert(0)
    }

    fn get_parameter_mode(&self, offset:usize) -> ParameterMode {
        match (self.memory[&self.program_counter] / (10i64.pow(offset as u32 + 1))) % 10 {
            0 => ParameterMode::Position,
            1 => ParameterMode::Immediate,
            2 => ParameterMode::Relative,
            _ => ParameterMode::Unknown,
        }
    }

    fn blocked(&self) -> bool {
        self.status == Status::Blocked
    }

    fn r#continue(&mut self) {
        if self.blocked() && self.inputs.len() > 0 {
            if self.live_feed {
                self.print_input();
            }
            self.status = Status::Running;
        }
        self.execute();
    }
}

fn run_program(program:&Vec<i64>, live:bool, instructions:&Vec<&str>) -> Option<i64> {
    let mut droid = Intcode::new(program.to_vec());
    droid.set_live_feed(live);
    droid.execute();
    if droid.blocked() {
        for instruction in instructions.iter() {
            droid.append_input(&instruction.chars().map(|c| c as i64).collect::<Vec<_>>());
            droid.input(10);
        }
    }
    droid.r#continue();
    if droid.outputs.len() > 0 && droid.outputs.get(droid.outputs.len() - 1).unwrap() > &127 {
        droid.outputs.pop_back()
    } else {
        None
    }

}

fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let args: Vec<String> = env::args().collect();
    let video_feed = args.len() == 2 && &args[1] == "y";
    let program: Vec<i64> = data.split(',')
                                    .map(|s| s.parse().unwrap())
                                    .collect();

    let instructions = vec![
        // Part 1: J = (!A and D) | (!C and D)
        vec!["NOT A J", "AND D J", "NOT C T", "AND D T", "OR T J", "WALK"],
        // Part 2: J =  (!A and D) | (!B and D) | (!C and D and H)
        vec!["NOT A J", "AND D J", "NOT B T", "AND D T", "OR T J", "NOT C T", "AND D T", "AND H T", "OR T J", "RUN"]
    ];

    for (part, instructions) in instructions.iter().enumerate() {
        if let Some(result) = run_program(&program, video_feed, instructions) {
            println!("Part {:?}: {:?} amount of hull damage is reported by the springdroid\n", part + 1, result);
        } else {
            println!("Part {:?} did not complete\n", part + 1);
        }
    }
}
