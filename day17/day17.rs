use std::fs;
use std::u64;
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

    fn append_input(&mut self, values:Vec<i64>) {
        self.inputs.append(&mut VecDeque::from(values.clone()));
    }

    fn execute(&mut self) {
        while self.status == Status::Running {
            self.tick();
        }
    }

    fn print_output(&mut self) {
        print!("{}", self.outputs.iter().map(|&c| c as u8 as char).collect::<String>());
        self.outputs.clear();
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
}

type Coordinates = (i32, i32);
type AreaMap = HashMap<Coordinates, u8>;
const ROW_NUM:[i32; 4] = [-1, 0, 1, 0];
const COL_NUM:[i32; 4] = [ 0, 1, 0,-1];

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
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

    fn from_value(&self, v:u8) -> Option<Direction> {
        match v {
            b'^' => Some(Direction::Up),
            b'>' => Some(Direction::Right),
            b'v' => Some(Direction::Down),
            b'<' => Some(Direction::Left),
            _   => None
        }
    }

    fn get_position(&self, pos:&Coordinates, dir:Direction) -> Coordinates {
        match self {
            Direction::Up => {
                match dir {
                    Direction::Right => (pos.0 + 1, pos.1),
                    Direction::Left => (pos.0 - 1, pos.1),
                    _ => unreachable!()
                }
            },
            Direction::Right => {
                match dir {
                    Direction::Down => (pos.0, pos.1 + 1),
                    Direction::Up => (pos.0, pos.1 - 1),
                    _ => unreachable!()
                }
            },
            Direction::Down => {
                match dir {
                    Direction::Right => (pos.0 + 1, pos.1),
                    Direction::Left => (pos.0 - 1, pos.1),
                    _ => unreachable!()
                }
            },
            Direction::Left => {
                match dir {
                    Direction::Up => (pos.0, pos.1 - 1),
                    Direction::Down => (pos.0, pos.1 + 1),
                    _ => unreachable!()
                }
            },
        }
    }
}

fn print_map(map:&AreaMap) {
    let (x_coords, y_coords): (Vec<_>, Vec<_>) = map.keys().cloned().unzip();
    let &x_max = x_coords.iter().max().unwrap();
    let &y_max = y_coords.iter().max().unwrap();

    for y in 0..y_max + 1 {
        let mut row:Vec<char> = Vec::new();
        for x in 0..x_max + 1 {
            if let Some(area) = map.get(&(x, y)) {
                row.push(*area as char);
            } else {
                row.push(' ');
            }
        }
        let line:String = row.iter().collect();
        println!("{}", line);
    }
}

fn get_alignment_parameters(map:&AreaMap) -> u64 {
    map.iter().filter(|&((x, y), v)| {
        if *v == b'#' {
            for i in 0..4 {
                if let Some(s) = map.get(&(x + COL_NUM[i], y + ROW_NUM[i])) {
                    if *s != b'#' {
                        return false
                    }
                } else {
                    return false
                }
            }
            return true
        }
        return false
    }).fold(0, |acc, ((x, y), _)| acc + x * y) as u64
}

fn get_turn(pos:&Coordinates, direction:&Direction, map:&AreaMap) -> Option<(Coordinates, Direction, u8)> {
    let pos_r = direction.get_position(pos, direction.right());
    let dir_r = direction.right();
    let pos_l = direction.get_position(pos, direction.left());
    let dir_l = direction.left();

    if let Some(&p) = map.get(&(pos_r)) {
        if p == b'#' {
            return Some((pos_r, dir_r, b'R'))
        } else if let Some(&p) = map.get(&(pos_l)) {
            if p == b'#' {
                return Some((pos_l, dir_l, b'L'))
            } else {
                return None
            }
        }
    } else if let Some(&p) = map.get(&(pos_l)) {
        if p == b'#' {
            return Some((pos_l, dir_l, b'L'))
        } else {
            return None
        }
    }
    return None
}

fn follow_direction(pos:&Coordinates, dir:&Direction, map:&AreaMap) -> (Coordinates, u8) {
    let directions:HashMap<Direction, (i32, i32)> =
        [(Direction::Up,    ( 0, -1)),
         (Direction::Right, ( 1,  0)),
         (Direction::Down,  ( 0,  1)),
         (Direction::Left,  (-1,  0))]
         .iter().cloned().collect();

    let (dx, dy) = directions.get(dir).unwrap();
    let mut steps = 1;

    loop {
        if let Some(&p) = map.get(&(pos.0 + dx * steps, pos.1 + dy * steps)) {
            if p != b'#' {
                break;
            }
        } else {
            break;
        }
        steps += 1;
    }

    ((pos.0 + dx * (steps - 1), pos.1 + dy * (steps - 1)), steps as u8)
}

fn follow_path(start_pos:&Coordinates, start_dir:&Direction, map:&AreaMap) -> Vec<u8> {
    let mut path:Vec<u8> = Vec::new();
    let (mut pos, mut dir, mut turn) = get_turn(start_pos, start_dir, map).unwrap();

    loop {
        let (new_pos, steps) = follow_direction(&pos, &dir, map);
        if steps > 0 {
            path.push(turn);

            if steps > 9 {
                path.push(char::from_digit((steps / 2) as u32, 10).unwrap() as u8);
                path.push(char::from_digit((steps / 2 + steps % 2) as u32, 10).unwrap() as u8);
            } else {
                path.push(char::from_digit(steps as u32, 10).unwrap() as u8);
            }

            match get_turn(&new_pos, &dir, map) {
                Some((p, d, t)) => {
                    pos = p;
                    dir = d;
                    turn = t;
                },
                None => break
            };
        } else {
            break;
        }
    }

    path
}

fn find_patterns(step:u8, max_depth:u8, path:String, patterns:&mut Vec<String>) -> Vec<String> {
    if path.len() == 0 {
        return patterns.to_vec()
    } else if step > max_depth {
        return Vec::new()
    }

    for i in (1..11.min(path.len())).rev() {
        let pattern = &path[..i];
        let last_char = pattern.chars().rev().nth(0).unwrap();
        if last_char == 'R' || last_char == 'L' {
            continue;
        }
        if path.matches(pattern).count() > 0 {
            patterns.push(pattern.to_string());

            let patterns_ = find_patterns(step + 1, max_depth, path.replace(&pattern, ""), patterns);
            if patterns_.len() == max_depth as usize {
                return patterns_
            } else {
                patterns.pop();
            }
        }
    }
    if patterns.len() == max_depth as usize {
        patterns.to_vec()
    } else {
        Vec::new()
    }
}

fn string_to_intcode_input(string:String) -> Vec<i64> {
    let mut input = string.chars().map(|c| c.to_string()).collect::<Vec<String>>().join(&",");
    input.push('\n');
    input.chars().map(|c| c as i64).collect::<Vec<_>>()
}

fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let args: Vec<String> = env::args().collect();
    let print = args.len() == 2 && &args[1] == "1";
    let video_feed = args.len() == 3 && &args[2] == "y";
    let program: Vec<i64> = data.split(',')
                                    .map(|s| s.parse().unwrap())
                                    .collect();
    let mut map = AreaMap::new();
    let mut robot = Intcode::new(program.clone());

    robot.execute();

    let mut x = 0;
    let mut y = 0;
    let mut droid:Coordinates = (0,0);
    let mut direction = Direction::Up;

    for c in robot.outputs {
        if c == 10 {
            y += 1;
            x = 0;
            continue;
        }

        if let Some(dir) = direction.from_value(c as u8) {
            droid = (x, y);
            direction = dir;
        }

        map.insert((x, y), c as u8);

        x += 1;
    }

    if print {
        print_map(&map);
    }

    // Part 1
    let alignment = get_alignment_parameters(&map);
    println!("The sum of the alignment parameters is {:?}", alignment);

    // Part 2
    let path = follow_path(&droid, &direction, &map);
    let mut movement_routine = path.iter().map(|&c| c as char).collect::<String>();
    let mut movement_functions = find_patterns(1, 3, movement_routine.clone(), &mut Vec::new());
    let mut robot_inputs:Vec<String> = Vec::new();

    for (i, &c) in b"ABC".iter().enumerate() {
        movement_routine = movement_routine.replace(&movement_functions[i], &(c as char).to_string());
    }
    robot_inputs.push(movement_routine);
    robot_inputs.append(&mut movement_functions);

    if video_feed {
        robot_inputs.push("y".to_string());
    } else {
        robot_inputs.push("n".to_string());
    }

    let mut prog = program.clone();
    prog[0] = 2;
    robot = Intcode::new(prog);
    robot.set_live_feed(video_feed);
    for i in robot_inputs {
        robot.append_input(string_to_intcode_input(i));
    }
    robot.execute();
    println!("The vacuum robot has collected {:?} dust", robot.outputs.pop_back().expect("NaN"));
}
