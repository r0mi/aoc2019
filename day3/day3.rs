use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

fn hamming_distance(x:i32, y:i32) -> i32 {
    x.abs() + y.abs()
}

fn main() {
    let f = File::open("input.txt").expect("Unable to open file");
    let mut reader = BufReader::new(f);
    let mut wire_map:HashMap<(i32, i32), char> = HashMap::new();
    let mut intersection_distances:Vec<i32> = Vec::new();
    let mut intersection_steps:Vec<i32> = Vec::new();
    let mut left_wire_position_steps:HashMap<(i32, i32), i32> = HashMap::new();
    let mut right_wire_position_steps:HashMap<(i32, i32), i32> = HashMap::new();

    let mut line = String::new();
    let _len = reader.read_line(&mut line).expect("Unable to read line");

    let mut x:i32 = 0;
    let mut y:i32 = 0;
    let mut dx:i32;
    let mut dy:i32;

    let mut step_counter = 0;
    for path in line.trim_end().split(',') {
        let mut iter = path.chars();
        let move_dir = iter.next();
        let steps = iter.collect::<String>().parse::<i32>().expect("NaN");
        match move_dir {
            Some('U') => {
                dy = 1;
                dx = 0;
            },
            Some('R') => {
                dy = 0;
                dx = 1;
            },
            Some('D') => {
                dy = -1;
                dx = 0;
            },
            Some('L') => {
                dy = 0;
                dx = -1;
            },
            Some(_) => panic!("unknwon direction"),
            None => panic!("unknwon direction"),
        }
        for _i in 0..steps {
            step_counter += 1;
            x += dx;
            y += dy;

            wire_map.insert((x, y), 'L');

            if !left_wire_position_steps.contains_key(&(x, y)) {
                left_wire_position_steps.insert((x, y), step_counter);
            }

        }
    }

    let mut line = String::new();
    let _len = reader.read_line(&mut line).expect("Unable to read line");
    let line = line.trim_end();

    x = 0;
    y = 0;
    step_counter = 0;
    for path in line.trim_end().split(',') {
        let mut iter = path.chars();
        let move_dir = iter.next();
        let steps = iter.collect::<String>().parse::<i32>().expect("NaN");
        match move_dir {
            Some('U') => {
                dy = 1;
                dx = 0;
            },
            Some('R') => {
                dy = 0;
                dx = 1;
            },
            Some('D') => {
                dy = -1;
                dx = 0;
            },
            Some('L') => {
                dy = 0;
                dx = -1;
            },
            Some(_) => panic!("unknwon direction"),
            None => panic!("unknwon direction"),
        }
        for _i in 0..steps {
            step_counter += 1;
            x += dx;
            y += dy;

            if !right_wire_position_steps.contains_key(&(x, y)) {
                right_wire_position_steps.insert((x, y), step_counter);
            }

            if !wire_map.contains_key(&(x, y)) {
                wire_map.insert((x, y), 'R');
            } else if wire_map[&(x, y)] == 'R' {
            } else {
                intersection_distances.push(hamming_distance(x, y));
                intersection_steps.push(step_counter + left_wire_position_steps[&(x, y)]);
                *wire_map.entry((x, y)).or_insert('X') = 'X';
            }

        }
    }

    println!("Closest intersection {:?}", intersection_distances.iter().min().unwrap());
    println!("Least steps intersection {:?}", intersection_steps.iter().min().unwrap());

}