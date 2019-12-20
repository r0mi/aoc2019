use std::fs;
use std::u64;
use std::env;
use std::usize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::BinaryHeap;

type Coordinates = (i32, i32);
type AreaMap = HashMap<Coordinates, char>;
type ItemLocations = HashMap<char, Coordinates>;
type VisitedLocations = HashMap<Coordinates, u64>;
type VisitedLevelLocations = HashMap<(Coordinates, u64), u64>;
type CameFrom = HashMap<Coordinates, Coordinates>;
type Path = Vec<Coordinates>;
type Neighbours = Vec<(Coordinates, u64, u64)>;
const ROW_NUM:[i32; 4] = [-1, 0, 1, 0];
const COL_NUM:[i32; 4] = [ 0, 1, 0,-1];

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Element {
    distance: u64,
    location: Coordinates,
    level: u64
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for Element {
    fn cmp(&self, other: &Element) -> Ordering {
        // Notice that the we flip the ordering on priority.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other.distance.cmp(&self.distance)
            .then_with(|| self.location.cmp(&other.location))
            .then_with(|| self.level.cmp(&other.level))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for Element {
    fn partial_cmp(&self, other: &Element) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn reconstruct_path(came_from:&CameFrom, target:&Coordinates) -> Path {
    let mut current = target;
    let mut total_path = [*current].to_vec();

    while came_from.contains_key(&current) {
        current = came_from.get(&current).unwrap();
        total_path.push(*current);
    }

    return total_path
}

fn shortest_path1(from:&Coordinates, to:&Coordinates, steps:u64, map:&AreaMap, visited:&mut VisitedLocations, came_from:&mut CameFrom, x_max:i32, y_max:i32, outer_doors:&ItemLocations, inner_doors:&ItemLocations) -> u64 {
    let mut min_steps = u64::MAX;

    visited.insert(*from, steps);

    for i in 0..4 {
        let mut neighbour = (from.0 + COL_NUM[i], from.1 + ROW_NUM[i]);
        if &neighbour == to {
            min_steps = steps + 1;
            came_from.insert(*from, *to);
        } else {
            let c = map.get(&neighbour);
            match c {
                Some('.') => {
                    if !visited.contains_key(&neighbour) || *visited.get(&neighbour).unwrap() > steps + 1 {
                        let v = shortest_path1(&neighbour, to, steps + 1, map, visited, came_from, x_max, y_max, outer_doors, inner_doors);
                        if v < min_steps {
                            came_from.insert(*from, neighbour);
                        }
                        min_steps = min_steps.min(v);
                    }
                },
                Some('#') => {},
                Some(' ') => {},
                Some(door) => {
                    if !visited.contains_key(&neighbour) || *visited.get(&neighbour).unwrap() > steps + 2 {
                        if is_outer_door(&neighbour, x_max, y_max) {
                            neighbour = *inner_doors.get(door).unwrap();
                        } else {
                            neighbour = *outer_doors.get(door).unwrap();
                        }
                    }
                    if !visited.contains_key(&neighbour) || *visited.get(&neighbour).unwrap() > steps + 2 {
                        let v = shortest_path1(&neighbour, to, steps + 2, map, visited, came_from, x_max, y_max, outer_doors, inner_doors);
                        if v < min_steps {
                            came_from.insert(*from, neighbour);
                        }
                        min_steps = min_steps.min(v);
                    }
                },
                None => {},
            }
        }
    }

    min_steps
}

fn get_neighbours(location:&Coordinates, steps:u64, level:u64, map:&AreaMap, outer_doors:&ItemLocations, inner_doors:&ItemLocations, x_max:i32, y_max:i32, start:&char, finish:&char, recursive:bool) -> Neighbours {
    let mut neighbours = Neighbours::new();
    for i in 0..4 {
        let mut pos = (location.0 + COL_NUM[i], location.1 + ROW_NUM[i]);
        let c = map.get(&pos);
        match c {
            Some('.') => {
                neighbours.push((pos, steps + 1, level));
            },
            Some('#') => {
                // Wall ignore
            },
            Some(' ') => {
                // Empty space, ignore
            },
            Some(door) => {
                // door
                let outer = is_outer_door(&pos, x_max, y_max);
                if !recursive {
                    if door == start || door == finish {
                        neighbours.push((pos, steps + 1, level));
                    } else if outer {
                        pos = *inner_doors.get(door).unwrap();
                        neighbours.push((pos, steps + 2, level));
                    } else {
                        pos = *outer_doors.get(door).unwrap();
                        neighbours.push((pos, steps + 2, level));
                    }
                } else {
                    if (door == start || door == finish) && level > 0 {
                        // do nothing, these are walls
                    } else if (door == start || door == finish) && level == 0 {
                        neighbours.push((pos, steps + 1, level));
                    } else if level == 0 && outer && door != start && door != finish {
                        // on this level these are not doors
                    } else if outer {
                        // or level n && outer door
                        pos = *inner_doors.get(door).unwrap();
                        neighbours.push((pos, steps + 2, level - 1));

                    } else {
                        // level n && inner door
                        pos = *outer_doors.get(door).unwrap();
                        neighbours.push((pos, steps + 2, level + 1));
                    }
                }
            },
            None => {}
        }
    }
   neighbours
}

fn shortest_path2(start:char, end:char, map:&AreaMap, outer_doors:&ItemLocations, inner_doors:&ItemLocations, x_max:i32, y_max:i32, recursive:bool) -> u64 {
    let mut priority_queue = BinaryHeap::new();
    priority_queue.push(Element { distance: 0, location: *outer_doors.get(&start).unwrap(), level: 0 });
    let mut visited = VisitedLevelLocations::new();
    let finish = outer_doors.get(&end).unwrap();

    while let Some(Element { distance: current_distance, location: current_location, level: current_level }) = priority_queue.pop() {
        if current_level == 0 && current_location == *finish {
            return current_distance;
        }
        for (next_location, next_distance, next_level) in get_neighbours(&current_location, current_distance, current_level, &map, &outer_doors, &inner_doors, x_max, y_max, &start, &end, recursive).iter() {
            if !visited.contains_key(&(*next_location, *next_level)) || *visited.get(&(*next_location, *next_level)).unwrap() > *next_distance {
                *visited.entry((*next_location, *next_level)).or_insert(0) = *next_distance;
                priority_queue.push(Element { distance: *next_distance, location: *next_location, level: *next_level});
            }
        }
    }
    u64::MAX
}

fn get_door(x:i32, y:i32, c:char, map:&AreaMap) -> Option<(Coordinates, String)> {
    let mut door:Vec<char> = vec![c];
    let mut loc:Option<Coordinates> = None;

    for j in 1..3 {
        for i in 0..4 {
            let pos = (x + COL_NUM[i] * j, y + ROW_NUM[i] * j);
            if let Some(c) = map.get(&pos) {
                match c {
                    '.' => {
                        if loc == None {
                            // The first accessed open ground is the correct door location
                            loc = Some((pos.0 - 2, pos.1 - 2));
                        }
                    },
                    'A'..='Z' => {
                        if (pos.0 - x).abs() < 2 && (pos.1 - y).abs() < 2 && (pos.0 > x || pos.1 > y) {
                            door.push(*c);
                        }
                    },
                    _ => {},
                }
            }
        }
    }

    if door.len() == 1 || loc == None {
        None
    } else {
        Some((loc.unwrap(), door.iter().collect()))
    }
}

fn is_outer_door(pos:&Coordinates, x_max:i32, y_max:i32) -> bool {
    pos.0 == 0 || pos.0 == x_max || pos.1 == 0 || pos.1 == y_max
}

fn find_doors(x_max:i32, y_max:i32, map:&AreaMap, outer_doors:&mut ItemLocations, inner_doors:&mut ItemLocations) -> (char, char) {
    let mut old_to_new:HashMap<String, char> = HashMap::new();
    let mut index:u8 = 0;

    for y in 0..y_max + 1 {
        for x in 0..x_max + 1 {
            let c = map.get(&(x, y));
            match c {
                Some('A'..='Z') => {
                    if let Some((pos, door)) = get_door(x, y, *c.unwrap(), map) {
                        let d:char;
                        if !old_to_new.contains_key(&door) {
                            if (b'A' + index as u8) > b'Z' {
                                d = (b'a' + index as u8 - b'Z' + b'A' - 1) as char;
                            } else {
                                d = (b'A' + index as u8) as char;
                            }
                            old_to_new.insert(door.to_string(), d);
                            index += 1;
                        } else {
                            d = *old_to_new.get(&door).unwrap();
                        }
                        if is_outer_door(&pos, x_max - 4, y_max -4) {
                            outer_doors.insert(d, pos);
                        } else {
                            inner_doors.insert(d, pos);
                        }
                    }
                },
                Some(_) => {},
                None => {},
            }
        }
    }
    (*old_to_new.get("AA").unwrap(), *old_to_new.get("ZZ").unwrap())
}

fn print_map(map:&AreaMap, path:&Vec<Coordinates>) {
    let (x_coords, y_coords): (Vec<_>, Vec<_>) = map.keys().cloned().unzip();
    let &x_max = x_coords.iter().max().unwrap();
    let &y_max = y_coords.iter().max().unwrap();
    for y in 0..y_max + 1 {
        let mut row:Vec<char> = Vec::new();
        for x in 0..x_max + 1 {
            if let Some(c) = map.get(&(x, y)) {
                if *c == '.' && path.contains(&(x, y)) {
                    row.push('█');
                } else {
                    row.push(*c);
                }
            }
        }
        let line:String = row.iter().collect();
        println!("{}", line);
    }
}

fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let args: Vec<String> = env::args().collect();
    let print = args.len() == 2 && &args[1] == "1";
    let lines: Vec<_> = data.split('\n').collect();
    let mut input:AreaMap = AreaMap::new();
    let mut map:AreaMap = AreaMap::new();
    let mut outer_doors:ItemLocations = ItemLocations::new();
    let mut inner_doors:ItemLocations = ItemLocations::new();
    let mut y_max:i32 = lines.len() as i32 - 1;
    let mut x_max:i32 = lines[0].len() as i32;

    for (y, line) in lines.iter().enumerate() {
        for (x, c) in line.chars().enumerate() {
            if x as i32 > x_max {
                x_max = x as i32;
            }
            input.insert((x as i32, y as i32), c);
            if x > 1 && y > 1 && y < y_max as usize {
                match c {
                    '.'|'#'|' ' => {
                        map.insert((x as i32 - 2, y as i32 - 2), c);
                    },
                    _ => {
                        map.insert((x as i32 - 2, y as i32 - 2), ' ');
                    }
                };
            }
        }
    }

    let (new_start_door, new_finish_door) = find_doors(x_max, y_max, &input, &mut outer_doors, &mut inner_doors);

    // Adjust for actual map
    x_max -= 4;
    y_max -= 4;

    for (k, v) in outer_doors.iter() {
        *map.entry(*v).or_insert('?') = *k;
    }
    for (k, v) in inner_doors.iter() {
        *map.entry(*v).or_insert('?') = *k;
    }

    let answer1:u64;
    if print {
        let mut came_from = CameFrom::new();
        answer1 = shortest_path1(outer_doors.get(&new_start_door).unwrap(), outer_doors.get(&new_finish_door).unwrap(), 0, &map, &mut VisitedLocations::new(), &mut came_from, x_max, y_max, &outer_doors, &inner_doors);
        let path = reconstruct_path(&came_from, outer_doors.get(&new_start_door).unwrap());
        print_map(&map, &path);
    } else {
        answer1 = shortest_path2(new_start_door, new_finish_door, &map, &outer_doors, &inner_doors, x_max, y_max, false);
    }

    println!("Part 1: It takes {:?} steps to get from the open tile marked AA to the open tile marked ZZ", answer1);
    let answer2 = shortest_path2(new_start_door, new_finish_door, &map, &outer_doors, &inner_doors, x_max, y_max, true);
    println!("Part 2: It takes {:?} steps to get from the open tile marked AA to the open tile marked ZZ, both at the outermost layer", answer2);
}