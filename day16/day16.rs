use std::char;

fn main() {
    const BASE_PATTERN:[i32; 4] = [0, 1, 0, -1];

    let input = include_str!("input.txt").trim_end();
    let initial_signal = input.chars().map(|s| s.to_string().parse::<i32>().unwrap()).collect::<Vec<i32>>();
    let offset: usize = input[..7].parse().unwrap();

    let mut signal = initial_signal.clone();

    for _ in 0..100 {
        for i in 0..signal.len() {
            let mut d = 0;
            for j in i..signal.len() {
                d += signal[j] * BASE_PATTERN[((j + 1) / (i + 1)) % 4];
            }
            signal[i] = d.abs() % 10;
        }
    }

    let output:String = signal.iter().map(|d| char::from_digit(*d as u32, 10).unwrap()).collect::<String>();
    println!("The first eight digits in the final output list are {}", &output[0..8]);

    // If you observe how the repeating pattern changes with increasing position
    // you will notice that the coefficents for elements len/2..len will
    // always be 1s, however for elements 0..len/2 it will be changing
    // If the actual offset is past len/2 then it will be depend on values from that
    // point forward and we can skip the values before the offset

    if offset > initial_signal.len() * 10000 / 2 {
        let mut real_signal = initial_signal.clone().into_iter().cycle().take(initial_signal.len() * 10000).skip(offset).collect::<Vec<i32>>();

        for _ in 0..100 {
            let mut sum = real_signal.iter().sum::<i32>();

            for value in &mut real_signal {
                let tmp = sum;
                sum -= *value;
                *value = tmp.abs() % 10;
            }
        }
        let output:String = real_signal.iter().map(|d| char::from_digit(*d as u32, 10).unwrap()).collect::<String>();
        println!("The eight-digit message embedded in the final output list is {}", &output[0..8]);
    } else {
        println!("Part 2 not implemented for this offset");
    }
}