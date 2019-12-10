use std::fs;

fn intcode(mut program:Vec<i32>, noun:i32, verb:i32) -> i32 {
    program[1] = noun;
    program[2] = verb;

    for x in (0..program.len()).step_by(4) {
        match program[x] {
            1 => {
                let save_idx:usize = program[x + 3] as usize;
                program[save_idx] = program[program[x + 1] as usize] + program[program[x + 2] as usize];
            },
            2 => {
                let save_idx:usize = program[x + 3] as usize;
                program[save_idx] = program[program[x + 1] as usize] * program[program[x + 2] as usize];
            },
            99 => break,
            _ => panic!("Unknown opcode {:?}", program[x]),
        }
    }
    program[0]
}

fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let program: Vec<i32> = data.split(',')
                                    .map(|s| s.parse().unwrap())
                                    .collect();

    println!("Value at position 0 after program halt is {}", intcode(program.clone(), 12, 2));

    for noun in 0..99 {
        for verb in 0..99 {
            let result = intcode(program.clone(), noun, verb);
            if result == 19690720 {
                println!("100 * noun {:?} + verb {:?} = {:?}", noun, verb, 100 * noun + verb);
                return;
            }
        }
    }

}