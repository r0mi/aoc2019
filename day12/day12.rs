fn gcd(a:u64, b:u64) -> u64 {
    // Greatest common divisor
    if b == 0 {
        return a
    } else {
        return gcd(b, a % b)
    }
}

fn lcm(a:u64, b:u64) -> u64 {
    // Lowest common multiple
    a * b / gcd(a, b)
}

fn get_coords(input_str:&str) -> Option<Vec<i32>> {
    let values:Vec<i32> = input_str[1..input_str.len() - 1]
        .split(", ")
        .map(|x| *&x[2..].parse::<i32>().unwrap())
        .collect::<Vec<i32>>();
    if values.len() != 3 {
        None
    } else {
        Some(values)
    }
}


fn main() {
    let lines = include_str!("input.txt").trim_right().lines().collect::<Vec<_>>();
    let mut positions:Vec<Vec<i32>> = Vec::new();
    let mut velocities:Vec<Vec<i32>> = Vec::new();

    for line in lines {
        if let Some(coords) = get_coords(line) {
            positions.push(coords);
            velocities.push(vec![0, 0, 0]);
        } else {
            panic!("Bad data");
        }
    }

    let initial_positions = positions.clone();
    let initial_velocities = velocities.clone();
    let mut axis_state_repeat_steps = vec![0, 0, 0];

    for step in 1.. {
        // Calculate new velocity vectors
        for i in 0..positions.len() - 1 {
            for j in i + 1..positions.len() {
                for coord in 0..3 {
                    if positions[i][coord] < positions[j][coord] {
                        velocities[i][coord] += 1;
                        velocities[j][coord] -= 1;
                    } else if positions[i][coord] > positions[j][coord] {
                        velocities[i][coord] -= 1;
                        velocities[j][coord] += 1;
                    }
                }

            }
        }

        // Calculate new positions
        for i in 0..positions.len() {
            for coord in 0..3 {
                positions[i][coord] += velocities[i][coord]
            }
        }

        if step == 1000 {
            let mut energy = 0;

            for (pos, vel) in positions.iter().zip(velocities.iter()) {
                energy += pos.iter().fold(0, |acc, v| acc + v.abs()) * vel.iter().fold(0, |acc, v| acc + v.abs())
            }

            println!("The total energy in the system after {:?} steps is {:?}", step, energy);
        }

        // Check for axis state repeats
        for axis in 0..3 {
            if axis_state_repeat_steps[axis] != 0 {
                break;
            }

            let mut found = true;

            for i in 0..positions.len() {
                if positions[i][axis] != initial_positions[i][axis] || velocities[i][axis] != initial_velocities[i][axis] {
                    found = false;
                    break;
                }
            }

            if found {
                axis_state_repeat_steps[axis] = step;
            }
        }

        if axis_state_repeat_steps.iter().all(|v| v > &0) {
            break;
        }
    }

    let lcm_xy = lcm(axis_state_repeat_steps[0], axis_state_repeat_steps[1]);
    let lcm_xyz = lcm(lcm_xy, axis_state_repeat_steps[2]);

    println!("It takes {:?} steps to reach a previous state", lcm_xyz);
}