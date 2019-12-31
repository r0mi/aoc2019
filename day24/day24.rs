use std::fs;
use std::env;
use std::collections::HashMap;
use std::collections::HashSet;
// use std::collections::BTreeMap; // With sorted keys, but slower

type Coordinates = (i32, i32);
type AreaMap = HashMap<Coordinates, char>;
type LevelMap = HashMap<i32, AreaMap>;
const SIZE:i32 = 5;
const ROW_NUM:[i32; 4] = [-1, 0, 1, 0];
const COL_NUM:[i32; 4] = [ 0, 1, 0,-1];

fn print_map(map:&AreaMap, recurisve:bool) {
    for y in 0..SIZE {
        let mut row:Vec<char> = Vec::new();
        for x in 0..SIZE {
            if recurisve && x == 2 && y == 2 {
                row.push('?');
            } else {
                if let Some(c) = map.get(&(x, y)) {
                    row.push(*c);
                } else {
                    row.push('.');
                }
            }
        }
        let line:String = row.iter().collect();
        println!("{}", line);
    }
    println!("");
}

fn print_level_maps(levels:&LevelMap, min:i32, max:i32) {
    for lvl in min..max + 1 {
        println!("Depth {:?}:", lvl);
        let map = levels.get(&lvl).unwrap();
        print_map(&map, true);
    }
    println!("");
}

fn map_state(map:&AreaMap) -> String {
    map.iter().fold([' '; (SIZE * SIZE) as usize], |mut acc, (k, &v)| {
        acc[(k.0 + k.1 * 5) as usize] = v;
        return acc;
    }).iter().collect::<String>()
}

fn biodiversity_rating(map:&AreaMap) -> u64 {
    map.iter().fold(0, |acc, (k, &v)| {
        if v == '#' {
            acc + (1 << (k.1 * 5 + k.0))
        } else {
            acc
        }
    })
}

fn adjacent_bugs(pos:Coordinates, map:&AreaMap) -> i32 {
    let mut bugs = 0;
    for i in 0..4 {
        let neighbour = (pos.0 + COL_NUM[i], pos.1 + ROW_NUM[i]);
        match map.get(&neighbour) {
            Some('#') => {
                bugs += 1;
            },
            _ => {},
        }
    }
    bugs
}

fn recursive_adjacent_bugs(pos:Coordinates, level:i32, levels:&LevelMap, min_lvl:i32, max_lvl:i32) -> i32 {
    let mut bugs = 0;

    for i in 0..4 {
        let neighbour = (pos.0 + COL_NUM[i], pos.1 + ROW_NUM[i]);
        if neighbour == (2, 2) {
            if level != max_lvl {
                match levels.get(&(level + 1)) {
                    Some(map) => {
                        if pos.1 == 2 { // y
                            let x:i32;
                            if pos.0 == 1 {
                                x = 0;
                            } else { // pos.0 == 3
                                x = 4;
                            }
                            for y in 0..SIZE {
                                match map.get(&(x, y)) {
                                    Some('#') => {
                                        bugs += 1;
                                    },
                                    Some(_) => {}
                                    None => {},
                                }
                            }
                        } else { // pos.0 == 2, x
                            let y:i32;
                            if pos.1 == 1 {
                                y = 0;
                            } else { // pos.1 == 3
                                y = 4;
                            }
                            for x in 0..SIZE {
                                match map.get(&(x, y)) {
                                    Some('#') => {
                                        bugs += 1;
                                    },
                                    Some(_) => {}
                                    None => {},
                                }
                            }
                        }
                    },
                    None => {},
                }
            }
        } else if neighbour.0 == -1 || neighbour.0 == SIZE || neighbour.1 == -1 || neighbour.1 == SIZE {
            if level != min_lvl {
                let new_neighbour:Coordinates;
                if neighbour.1 == -1 {
                    new_neighbour = (2, 1);
                } else if neighbour.1 == SIZE {
                    new_neighbour = (2, 3);
                } else if neighbour.0 == -1 {
                    new_neighbour = (1, 2);
                } else { // neighbour.0 == SIZE
                    new_neighbour = (3, 2);
                }
                match levels.get(&(level - 1)) {
                    Some(map) => {
                        match map.get(&new_neighbour) {
                            Some('#') => {
                                bugs += 1;
                            },
                            Some(_) => {}
                            None => {},
                        }
                    },
                    None => {},
                }
            }
        } else {
            match levels.get(&level).unwrap().get(&neighbour) {
                Some('#') => {
                    bugs += 1;
                },
                Some(_) => {}
                None => {},
            }
        }
    }
    bugs
}

fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let args: Vec<String> = env::args().collect();
    let print = args.len() == 2 && &args[1] == "1";
    let lines: Vec<_> = data.split('\n').collect();
    let mut initial_map = AreaMap::new();
    let mut map:AreaMap;
    let mut map_states:HashSet<String> = HashSet::new();

    for (y, line) in lines.iter().enumerate() {
        for (x, c) in line.chars().enumerate() {
            initial_map.insert((x as i32, y as i32), c);
        }
    }
    map = initial_map.clone();

    // Part 1
    loop {
        let mut new_map = map.clone();
        for y in 0..SIZE {
            for x in 0..SIZE {
                let pos:Coordinates = (x, y);
                let adjacent_bugs = adjacent_bugs(pos, &map);
                match map.get(&pos) {
                    Some('#') => {
                        if adjacent_bugs != 1 {
                            *new_map.entry(pos).or_insert('.') = '.';
                        }
                    },
                    Some('.') | None => {
                        if adjacent_bugs == 1 || adjacent_bugs == 2 {
                            *new_map.entry(pos).or_insert('.') = '#';
                        }
                    },
                    _ => unreachable!(),
                }
            }
        }

        let state = map_state(&new_map);
        if map_states.contains(&state) {
            if print {
                print_map(&new_map, false);
            }
            println!("The biodiversity rating for the first layout that appears twice is {:?}", biodiversity_rating(&new_map));
            break;
        } else {
            map_states.insert(state);
        }
        map = new_map;
    }

    // Part 2
    let mut map_levels = LevelMap::new();
    map_levels.insert(-1, AreaMap::new());
    map_levels.insert(0, initial_map.clone());
    map_levels.insert(1, AreaMap::new());
    let mut min_lvl:i32 = -1;
    let mut max_lvl:i32 = 1;

    for i in 0.. {
        let mut new_map_levels = map_levels.clone();
        let min = min_lvl;
        let max = max_lvl + 1;

        for lvl in min..max {
            let mut new_map = new_map_levels.get(&lvl).unwrap().clone();
            for y in 0..SIZE {
                for x in 0..SIZE {
                    if x == 2 && y == 2 {
                        continue;
                    }
                    let pos:Coordinates = (x, y);
                    let neighbours = recursive_adjacent_bugs(pos, lvl, &map_levels, min_lvl, max_lvl);
                    match new_map_levels.get(&lvl).unwrap().get(&pos) {
                        Some('#') => {
                            if neighbours != 1 {
                                *new_map.entry(pos).or_insert('.') = '.';
                            }
                        },
                        Some('.') | None => {
                            if neighbours == 1 || neighbours == 2 {
                                *new_map.entry(pos).or_insert('.') = '#';
                            }
                        },
                        Some(_) => {unreachable!()},
                    }
                }
            }
            new_map_levels.insert(lvl, new_map);
        }
        let bugs_on_min_lvl = new_map_levels.get(&min_lvl).unwrap().iter().fold(0, |acc, (_, v)| if *v == '#' { acc + 1} else { acc });
        if bugs_on_min_lvl > 0 {
            min_lvl -= 1;
            new_map_levels.insert(min_lvl, AreaMap::new());
        }
        let bugs_on_max_lvl = new_map_levels.get(&max_lvl).unwrap().iter().fold(0, |acc, (_, v)| if *v == '#' { acc + 1} else { acc });
        if bugs_on_max_lvl > 0 {
            max_lvl += 1;
            new_map_levels.insert(max_lvl, AreaMap::new());
        }
        map_levels = new_map_levels;
        if i == 199 {
            if print {
                print_level_maps(&map_levels, min_lvl, max_lvl);
            }
            let mut bugs = 0;
            for lvl in min_lvl..max_lvl + 1 {
                bugs += map_levels.get(&lvl).unwrap().iter().fold(0, |acc, (_, v)| if *v == '#' { acc + 1} else { acc });
            }

            println!("{:?} bugs are present after 200 minutes", bugs);

            break;
        }
    }
}
