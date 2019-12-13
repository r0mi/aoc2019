use std::fs;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::{thread, time};
use std::env;

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
        }
    }

    fn input(&mut self, value:i64) {
        self.inputs.push_back(value);
    }

    fn execute(&mut self) {
        while self.status == Status::Running {
            self.tick();
        }
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

fn print_game_state(x_max:i32, y_max:i32, game:&HashMap<(i32, i32), u8>, ball:&(i32, i32), paddle:&(i32, i32)) {
    print!("{}[2J", 27 as char);
    for y in 0..y_max + 1 {
        let mut row:Vec<u8> = Vec::new();
        for x in 0..x_max + 1 {
            if &(x, y) == ball {
                row.push(4);
            } else if &(x, y) == paddle {
                row.push(3);
            } else {
                if let Some(panel) = game.get(&(x, y)) {
                    row.push(*panel);
                } else {
                    row.push(0);
                }
            }
        }
        let line:String = row.iter().map(|i| if i == &1 { "|||" } else if i == &2 { "███" } else if i == &3 { "‾‾‾" } else if i == &4 { " * " } else { "   " }).collect();
        println!("{}", line);
    }
    thread::sleep(time::Duration::from_millis(33));
}

fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let args: Vec<String> = env::args().collect();
    let mut program: Vec<i64> = data.split(',')
                                    .map(|s| s.parse().unwrap())
                                    .collect();
    let mut tiles:HashMap<(i32, i32), u8> = HashMap::new();
    let mut paddle:(i32, i32) = (0, 0);
    let mut ball:(i32, i32) = (0, 0);

    let mut arcade_cabinet = Intcode::new(program.clone());
    arcade_cabinet.execute();
    println!("There are {:?} block tiles are on the screen when the game exits", arcade_cabinet.outputs.iter().enumerate().fold(0, |acc, (i, x)| if (i + 1) % 3 == 0 && *x == 2 { acc + 1 } else { acc } ));

    program[0] = 2;
    arcade_cabinet = Intcode::new(program);
    let mut x_max:i32 = 0;
    let mut y_max:i32 = 0;
    let mut ball_direction = 0;
    let mut paddle_direction;
    let mut score = 0;
    let print = args.len() == 2 && &args[1] == "1";

    while !arcade_cabinet.finished() {
        if arcade_cabinet.blocked() {
            if ball.0 == paddle.0 {
                if paddle.1 - ball.1 == 2 {
                    paddle_direction = 0;
                } else {
                    paddle_direction = ball_direction;
                }
            } else if ball.0 < paddle.0 {
                paddle_direction = -1;
            } else {
                paddle_direction = 1;
            }
            arcade_cabinet.input(paddle_direction as i64);

            arcade_cabinet.r#continue();
        } else {
            arcade_cabinet.execute();
        }

        while arcade_cabinet.outputs.len() > 0 && arcade_cabinet.outputs.len() % 3 == 0 {
            let x = arcade_cabinet.outputs.pop_front().unwrap() as i32;
            let y = arcade_cabinet.outputs.pop_front().unwrap() as i32;
            let t = arcade_cabinet.outputs.pop_front().unwrap();

            if x == -1 && y == 0 {
                score = t;
            } else if t == 4 {
                if ball != (0, 0) {
                    ball_direction = x - ball.0;
                }
                ball = (x, y);
            } else if t == 3 {
                paddle = (x, y);
            } else {
                *tiles.entry((x, y)).or_insert(0) = t as u8;
            }
        }

        if x_max == 0 {
            let (x_coords, y_coords): (Vec<_>, Vec<_>) = tiles.keys().cloned().unzip();
            x_max = *x_coords.iter().max().unwrap();
            y_max = *y_coords.iter().max().unwrap();
        }

        if print {
            print_game_state(x_max, y_max, &tiles, &ball, &paddle);
        }
    }

    println!("The score after the last block is broken is {:?}", score);
}