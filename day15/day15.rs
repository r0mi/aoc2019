use std::fs;
use std::u64;
use std::env;
use std::usize;
// use std::cmp::Ordering;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::HashSet;
// use std::collections::BinaryHeap;

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

type Coordinates = (i32, i32);
type AreaMap = HashMap<Coordinates, u8>;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn right(&self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    fn left(&self) -> Direction {
        self.right().right().right()
    }

    fn get_value(&self) -> i64 {
        match self {
            Direction::North => 1,
            Direction::East => 4,
            Direction::South => 2,
            Direction::West => 3,
        }
    }
}

/*#[derive(Copy, Clone, Eq, PartialEq)]
struct Element {
    priority: u64,
    position: Coordinates,
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for Element {
    fn cmp(&self, other: &Element) -> Ordering {
        // Notice that the we flip the ordering on priority.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other.priority.cmp(&self.priority)
            .then_with(|| self.position.cmp(&other.position))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for Element {
    fn partial_cmp(&self, other: &Element) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Shortest path
fn a_star_search(start:&Coordinates, goal:&Coordinates, map:&AreaMap) -> u64 {
    fn heuristic_cost_estimate(a:&Coordinates, b:&Coordinates) -> u64 {
        (a.0 - b.0).abs() as u64 + (a.1-b.1).abs() as u64
    }

    fn direct_cost_estimate(b:&Coordinates, map:&AreaMap) -> u64 {
        match map.get(b) {
            Some(v) => {
                match v {
                    0 => u64::MAX,
                    _ => 1
                }
            }
            None => u64::MAX
        }
    }

    let mut priority_queue = BinaryHeap::new();
    priority_queue.push(Element { priority: 0, position: *start });

    let mut g_score:HashMap<Coordinates, u64> = HashMap::new();
    g_score.insert(*start, 0);

    let mut f_score:HashMap<Coordinates, u64> = HashMap::new();
    f_score.insert(*start, heuristic_cost_estimate(start, goal));

    let row_num = vec![-1, 0, 1, 0];
    let col_num = vec![ 0, 1, 0,-1];

    while let Some(Element { priority: _, position }) = priority_queue.pop() {
        if position == *goal {
            return *g_score.get(&position).unwrap()
        }

        for i in 0..4 {
            let neighbour = (position.0 + col_num[i], position.1 + row_num[i]);
            let neighbour_cost = direct_cost_estimate(&neighbour, map);
            let tentative_g_score = if neighbour_cost == u64::MAX {
                u64::MAX
            } else {
                g_score.get(&position).unwrap() + neighbour_cost
            };
            let g_score_neighbour = match g_score.get(&neighbour) {
                Some(v) => *v,
                None => u64::MAX
            };

            if tentative_g_score < g_score_neighbour {
                f_score.insert(neighbour, tentative_g_score + heuristic_cost_estimate(&neighbour, goal));
                if !g_score.contains_key(&neighbour) {
                    priority_queue.push(Element { priority: *f_score.get(&neighbour).unwrap(), position: neighbour});
                }
                g_score.insert(neighbour, tentative_g_score);
            }
        }
    }

    unreachable!()
}*/

fn longest_path(position:Coordinates, cost:u64, map:&AreaMap, visited:&mut HashSet<Coordinates>) -> u64 {
    let mut max_cost = cost;

    let row_num = vec![-1, 0, 1, 0];
    let col_num = vec![ 0, 1, 0,-1];

    visited.insert(position);

    for i in 0..4 {
        let neighbour = (position.0 + col_num[i], position.1 + row_num[i]);
        if let Some(v) = map.get(&neighbour) {
            if *v == 1 && !visited.contains(&neighbour) {
                max_cost = max_cost.max(longest_path(neighbour, cost + 1, map, visited));
            }
        }
    }

    max_cost
}

fn shortest_path(position:Coordinates, steps:u64, map:&AreaMap, visited:&mut HashSet<Coordinates>) -> u64 {
    let mut min_steps = u64::MAX;

    let row_num = vec![-1, 0, 1, 0];
    let col_num = vec![ 0, 1, 0,-1];

    visited.insert(position);

    for i in 0..4 {
        let neighbour = (position.0 + col_num[i], position.1 + row_num[i]);
        if let Some(v) = map.get(&neighbour) {
            if *v == 1 && !visited.contains(&neighbour) {
                min_steps = min_steps.min(shortest_path(neighbour, steps + 1, map, visited));
            } else if *v == 2 {
                min_steps = min_steps.min(steps + 1);
            }
        }
    }

    min_steps
}

fn wall_follower(map:&mut AreaMap, start:Coordinates, robot:&mut Intcode) -> Coordinates {
    let mut goal = (-1, -1);
    let (mut x, mut y) = start;

    let mut direction = Direction::North;
    map.insert(start, 1);

    robot.input(direction.get_value());

    loop {
        if robot.blocked() {
            robot.r#continue();
        } else {
            robot.execute();
        }

        let (x_, y_):Coordinates;

        match direction {
            Direction::North => {x_ = x; y_ = y - 1 },
            Direction::East => {x_ = x + 1; y_ = y },
            Direction::South => {x_ = x; y_ = y + 1 },
            Direction::West => {x_ = x - 1; y_ = y },
        };

        match robot.outputs.pop_front().unwrap() {
            0 => {
                map.entry((x_, y_)).or_insert(0);
                direction = direction.left();
            },
            1 => {
                map.entry((x_, y_)).or_insert(1);
                direction = direction.right();
                x = x_;
                y = y_;
            },
            2 => {
                map.entry((x_, y_)).or_insert(2);
                x = x_;
                y = y_;
                goal = (x, y);
            },
            _ => unreachable!(),
        }

        if (x, y) == start && direction == Direction::North {
            // Back at starting point & direction
            break;
        }

        robot.input(direction.get_value());
    }

    goal
}

fn print_map(map:&AreaMap, droid:&Coordinates) {
    let (x_coords, y_coords): (Vec<_>, Vec<_>) = map.keys().cloned().unzip();
    let (&x_min, &x_max) = (x_coords.iter().min().unwrap(), x_coords.iter().max().unwrap());
    let (&y_min, &y_max) = (y_coords.iter().min().unwrap(), y_coords.iter().max().unwrap());

    print!("{}[2J", 27 as char);
    for y in y_min..y_max + 1 {
        let mut row:Vec<u8> = Vec::new();
        for x in x_min..x_max + 1 {
            if &(x, y) == droid {
                row.push(3);
            } else if &(x, y) == &(0, 0) {
                row.push(255);
            } else {
                if let Some(area) = map.get(&(x, y)) {
                    row.push(*area);
                } else {
                    row.push(4);
                }
            }
        }
        let line:String = row.iter().map(|i| if i == &0 { "#" } else if i == &1 { "." } else if i == &2 { "O" } else if i == &3 { "D" } else if i == &255 { "X" } else { " " }).collect();
        println!("{}", line);
    }
}

fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let args: Vec<String> = env::args().collect();
    let print = args.len() == 2 && &args[1] == "1";
    let program: Vec<i64> = data.split(',')
                                    .map(|s| s.parse().unwrap())
                                    .collect();
    let mut map = AreaMap::new();
    let goal:Coordinates;

    goal = wall_follower(&mut map, (0, 0), &mut Intcode::new(program.clone()));

    if print {
        print_map(&map, &(255, 255));
    }

    let moves = shortest_path((0, 0), 0, &map, &mut HashSet::new());
    // let moves = a_star_search(&(0, 0), &goal, &map);
    println!("Fewest number of movement commands to move repair droid to oxygen system is {:?}", moves);

    let minutes = longest_path(goal, 0, &map, &mut HashSet::new());

    println!("It will take {:?} minutes to fill all locations with oxygen", minutes);
}

