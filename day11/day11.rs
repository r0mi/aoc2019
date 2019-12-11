use std::fs;
use std::collections::VecDeque;
use std::collections::HashMap;

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

#[derive(Debug)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn left(&self) -> Direction {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }

    fn right(&self) -> Direction {
        self.left().left().left()
    }
}

fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let program: Vec<i64> = data.split(',')
                                    .map(|s| s.parse().unwrap())
                                    .collect();
    let mut panels:HashMap<(i32, i32), u8> = HashMap::new();
    let (mut x, mut y) = (0, 0);
    let mut direction = Direction::Up;

    let mut painting_robot = Intcode::new(program.clone());
    while !painting_robot.finished() {
        let panel = panels.entry((x, y)).or_insert(0);
        painting_robot.input(*panel as i64);
        if painting_robot.blocked() {
            painting_robot.r#continue();
        } else {
            painting_robot.execute();
        }
        if painting_robot.outputs.len() == 2 {
            let color = painting_robot.outputs.pop_front().unwrap();
            let dir = painting_robot.outputs.pop_front().unwrap();
            *panel = color as u8;

            match dir {
                0 => direction = direction.left(),
                1 => direction = direction.right(),
                _ => panic!("Unknown direction"),
            };

            match direction {
                Direction::Up => y += 1,
                Direction::Right => x += 1,
                Direction::Down => y -= 1,
                Direction::Left => x -= 1,
            };

        } else {
            break;
        }
    }

    println!("{:?} panels have been painted at least once", panels.len());

    let mut panels:HashMap<(i32, i32), u8> = HashMap::new();
    let (mut x, mut y) = (0, 0);
    let mut direction = Direction::Up;
    panels.insert((0,0), 1);


    let mut painting_robot = Intcode::new(program.clone());
    while !painting_robot.finished() {
        let panel = panels.entry((x, y)).or_insert(0);
        painting_robot.input(*panel as i64);
        if painting_robot.blocked() {
            painting_robot.r#continue();
        } else {
            painting_robot.execute();
        }
        if painting_robot.outputs.len() == 2 {
            let color = painting_robot.outputs.pop_front().unwrap();
            let dir = painting_robot.outputs.pop_front().unwrap();
            *panel = color as u8;

            match dir {
                0 => direction = direction.left(),
                1 => direction = direction.right(),
                _ => panic!("Unknown direction"),
            };

            match direction {
                Direction::Up => y -= 1,
                Direction::Right => x += 1,
                Direction::Down => y += 1,
                Direction::Left => x -= 1,
            };
        } else {
            break;
        }
    }

    let (x_coords, y_coords): (Vec<_>, Vec<_>) = panels.keys().cloned().unzip();
    let (&x_min, &x_max) = (x_coords.iter().min().unwrap(), x_coords.iter().max().unwrap());
    let (&y_min, &y_max) = (y_coords.iter().min().unwrap(), y_coords.iter().max().unwrap());

    println!("The emergency hull painting robot paints the following registration identifier:");

    for y in y_min..y_max + 1 {
        let mut row:Vec<u8> = Vec::new();
        for x in x_min..x_max + 1 {
            let mut panel = panels.entry((x, y)).or_insert(0);
            row.push(*panel);
        }
        let line:String = row.iter().map(|i| if i == &1 { "██" } else { "  "}).collect();
        println!("{}", line);
    }
}