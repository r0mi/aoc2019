use std::fs;
use std::env;
use std::usize;
use std::cmp::PartialOrd;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::HashSet;

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
    verbose: bool
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
            verbose: false
        }
    }

    fn set_verbose(&mut self, verbose:bool) {
        self.verbose = verbose;
    }

    fn output_to_string(&self) -> String {
        self.outputs.iter().map(|&c| c as u8 as char).collect::<String>()
    }

    fn get_output_string(&mut self) -> String {
        if self.verbose {
            self.print_output();
        }
        let output = self.output_to_string();
        self.outputs.clear();
        output
    }

    fn append_input(&mut self, values:&Vec<i64>) {
        self.inputs.append(&mut VecDeque::from(values.to_vec()));
    }

    fn execute(&mut self) {
        if self.inputs.len() > 0 && self.verbose {
            self.print_input();
        }
        while self.status == Status::Running {
            self.tick();
        }
    }

    fn print_output(&self) {
        print!("{}", self.output_to_string());
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
            self.status = Status::Running;
        }
        self.execute();
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Action {
    Movement(Direction),
    Take(String),
    Drop(String),
    Inv,
}

impl Action {
    fn to_instruction(&self) -> Vec<i64> {
        let mut instruction:Vec<i64> = Vec::new();
        match self {
            Action::Movement(dir) => {
                instruction.append(&mut dir.to_string().chars().map(|c| c as i64).collect::<Vec<_>>());
            },
            Action::Take(thing) => {
                instruction.append(&mut "take ".chars().map(|c| c as i64).collect::<Vec<_>>());
                instruction.append(&mut thing.chars().map(|c| c as i64).collect::<Vec<_>>());
            },
            Action::Drop(thing) => {
                instruction.append(&mut "drop ".chars().map(|c| c as i64).collect::<Vec<_>>());
                instruction.append(&mut thing.chars().map(|c| c as i64).collect::<Vec<_>>());
            },
            Action::Inv => {
                instruction.append(&mut "inv".chars().map(|c| c as i64).collect::<Vec<_>>());
            },
        }
        instruction.push(10);
        instruction
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy, Ord, PartialOrd)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn to_string(&self) -> &str {
        match self {
            Direction::North => "north",
            Direction::East => "east",
            Direction::South => "south",
            Direction::West => "west",
        }
    }

    fn reverse(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy, Ord, PartialOrd)]
struct Coordinates(i32, i32);

impl Coordinates {
    fn get_position(&self, direction:Direction) -> Coordinates {
        match direction {
            Direction::North => {
                Coordinates(self.0, self.1 - 1)
            },
            Direction::East => {
                Coordinates(self.0 + 1, self.1)
            },
            Direction::South => {
                Coordinates(self.0, self.1 + 1)
            },
            Direction::West => {
                Coordinates(self.0 - 1, self.1)
            },
        }
    }

}

type AreaMap = HashMap<Coordinates, (String, Vec<Direction>, Option<Vec<String>>)>;
type Items = HashSet<String>;
type VisitedLocations = HashSet<Coordinates>;
type Path = Vec<Direction>;

fn parse_doors(output:&str) -> Option<Vec<Direction>> {
    let mut doors:Vec<Direction> = Vec::new();
    let mut door_list:bool = false;

    for line in output.lines() {
        if line.starts_with("- ") && door_list {
            let mut iter = line.split_ascii_whitespace();
            iter.next();

            match iter.next() {
                Some("east") => {
                    doors.push(Direction::East);
                },
                Some("north") => {
                    doors.push(Direction::North);
                },
                Some("west") => {
                    doors.push(Direction::West);
                },
                Some("south") => {
                    doors.push(Direction::South);
                },
                Some(_) => {
                    panic!("Unknown input {:?}", line);
                },
                None => unreachable!(),
            }
        } else if line.starts_with("Doors here lead:") {
            door_list = true;
        } else if !line.starts_with("- ") && door_list {
            doors.sort();
            return Some(doors);
        }
    }

    None
}

fn parse_items(output:&str) -> Option<Vec<String>> {
    let mut items:Vec<String> = Vec::new();
    let mut item_list:bool = false;

    for line in output.lines() {
        if line.starts_with("- ") && item_list {
            let mut iter = line.split("- ");
            iter.next();
            items.push(iter.next().unwrap().to_string());
        } else if line.starts_with("Items here:") || line.starts_with("Items in your inventory:") {
            item_list = true;
        } else if !line.starts_with("- ") && item_list {
            return Some(items);
        }
    }

    None
}

fn parse_room(output:&str) -> String {
    for line in output.lines() {
        if line.starts_with("== ") {
            return line[3..line.len() - 3].to_string()
        }
    }

    "".to_string()
}

fn find_path(from:Coordinates, to:Coordinates, map:&AreaMap, visited:&mut VisitedLocations, path:&mut Path) -> bool {
    visited.insert(from);
    if let Some((_, doors, _)) = map.get(&from) {
        for door in doors {
            let room = from.get_position(*door);
            if room == to {
                path.push(*door);
                return true;
            } else if !visited.contains(&room) {
                if find_path(room, to, map, visited, path) {
                    path.push(*door);
                    return true;
                }
            }
        }
    }

    false
}

fn navigate(droid:&mut Intcode, path:&Path, pos:&mut Coordinates) {
    for &dir in path {
        droid.append_input(&Action::Movement(dir).to_instruction());
        droid.r#continue();
        if !droid.output_to_string().contains("ejected") {
            *pos = pos.get_position(dir);
        }
    }
}

fn explore(droid:&mut Intcode, map:&mut AreaMap, pos:Coordinates, came_from:Direction, avoid_items:&mut Items, verbose:bool) -> bool {
    let output = droid.get_output_string();
    let doors = parse_doors(&output);
    let room = parse_room(&output);
    let items = parse_items(&output);
    let mut last_item:String = "".to_string();

    if !map.contains_key(&pos) {
        map.insert(pos, (room.clone(), doors.clone().unwrap(), items.clone()));

        // Pick up items on the go
        if let Some(items) = items.clone() {
            for item in items {
                if !avoid_items.contains(&item) {
                    droid.append_input(&Action::Take(item.clone()).to_instruction());
                    droid.r#continue();

                    droid.get_output_string();
                    last_item = item.clone();

                    if !droid.blocked() {
                        avoid_items.insert(item);
                        return true;
                    }
                }
            }
        }

        if let Some(doors) = doors.clone() {
            for &dir in &doors {
                if dir == came_from {
                    continue;
                }

                // Go through the door
                droid.append_input(&Action::Movement(dir).to_instruction());
                droid.r#continue();
                let output = droid.output_to_string();
                if output.contains("can't move") {
                    if verbose {
                        print!("{}", output);
                    }
                    avoid_items.insert(last_item);
                    return true;
                }
                if explore(droid, map, pos.get_position(dir), dir.reverse(), avoid_items, verbose) {
                    return true;
                }

                // Come back through the door
                droid.append_input(&Action::Movement(dir.reverse()).to_instruction());
                droid.set_verbose(false);
                droid.r#continue();
                droid.outputs.clear();
                droid.set_verbose(verbose);
            }
        }
    }

    return false;
}

fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let program: Vec<i64> = data.split(',')
                                    .map(|s| s.parse().unwrap())
                                    .collect();

    let args: Vec<String> = env::args().collect();
    let verbose = args.len() == 2 && &args[1] == "1";
    let mut map = AreaMap::new();
    let mut current_position = Coordinates(0, 0);
    let mut droid = Intcode::new(program.to_vec());
    let mut avoid_items = Items::new();

    droid.set_verbose(verbose);
    droid.execute();
    avoid_items.insert("infinite loop".to_string());

    while explore(&mut droid, &mut map, current_position, Direction::North, &mut avoid_items, verbose) {
        if verbose {
            println!("\n--------- Bad item, restarting exploration ---------");
        }
        droid = Intcode::new(program.to_vec());
        droid.set_verbose(verbose);
        droid.execute();
        map = AreaMap::new();
    }

    if verbose {
        println!("\n--------- Everything mapped out ---------");
    }

    droid.append_input(&Action::Inv.to_instruction());
    droid.r#continue();
    let inventory = parse_items(&droid.get_output_string()).unwrap();

    let security_checkpoint = map.iter().find(|(_, v)| v.0 == "Security Checkpoint").unwrap().0;
    let mut path = Path::new();
    find_path(current_position, *security_checkpoint, &map, &mut VisitedLocations::new(), &mut path);
    path.reverse();

    if verbose {
        println!("\n--------- Finding path to Security Checkpoint ---------");
        println!("Path {:?}", path);
        println!("\n--------- Navigating to Security Checkpoint ---------");
    }

    navigate(&mut droid, &path, &mut current_position);

    if verbose {
        println!("\n--------- Bruteforcing for correct weight ---------");
    }

    let pressure_sensitive_floor = map.iter().find(|(_, v)| v.0 == "Pressure-Sensitive Floor").unwrap().0;
    path = Path::new();
    find_path(current_position, *pressure_sensitive_floor, &map, &mut VisitedLocations::new(), &mut path);

    let prev_pos = current_position;

    for combination in (0..2_usize.pow(inventory.len() as u32)).rev() {
        let mut droid_clone = droid.clone();
        droid_clone.set_verbose(false);
        for j in 0..inventory.len() {
            if (combination >> j) & 1 == 0 {
                droid_clone.append_input(&Action::Drop(inventory[j].clone()).to_instruction());
                droid_clone.r#continue();
            }
        }
        droid_clone.outputs.clear();
        droid_clone.set_verbose(verbose);

        if verbose {
            droid_clone.append_input(&Action::Inv.to_instruction());
            droid_clone.r#continue();
            let _output = droid_clone.get_output_string();
        }

        navigate(&mut droid_clone, &path, &mut current_position);
        let output = droid_clone.get_output_string();
        if current_position == prev_pos {
            if verbose {
                println!("\n--------- Combination failed ---------\n");
            }
        } else {
            if !verbose {
                println!("{}", output);
            }
            break;
        }
    }
}
