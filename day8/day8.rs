use std::collections::HashMap;

fn main() {
    let input = include_str!("input.txt").trim_right();
    const IMAGE_WIDTH:usize = 25;
    const IMAGE_HEIGHT:usize = 6;
    let mut image:[[u8; IMAGE_WIDTH]; IMAGE_HEIGHT] = [[2; IMAGE_WIDTH]; IMAGE_HEIGHT];
    let mut histogram:HashMap<u8, u32> = HashMap::new();
    let mut layer_histograms:Vec<HashMap<u8, u32>> = Vec::new();

    let mut row = 0;
    let mut col = 0;

    for (i, c) in input.chars().enumerate() {
        if i != 0 && i % (IMAGE_HEIGHT * IMAGE_WIDTH) == 0 {
            layer_histograms.push(histogram.clone());
            histogram.clear();
            col = 0;
            row = 0;
        }

        let pixel:u8 = c.to_string().parse::<u8>().unwrap();
        *histogram.entry(pixel).or_insert(0) += 1;
        if pixel != 2 && image[row][col] == 2 {
            image[row][col] = pixel;
        }
        col += 1;
        if col % IMAGE_WIDTH == 0 {
            col = 0;
            row += 1;
        }
    }

    let mut min_zeros = (IMAGE_WIDTH * IMAGE_HEIGHT) as u32;
    let mut result:u32 = 0;

    for layer in layer_histograms {
        if let Some(zeros) = layer.get(&0) {
            if zeros < &min_zeros {
                min_zeros = *zeros;
                let ones = layer.get(&1).unwrap();
                let twos = layer.get(&2).unwrap();
                result = ones * twos;
            }
        } else {
            println!("{:?}", layer);
            println!("No zeros");
        }
    }
    println!("The number of 1 digits multiplied by the number of 2 digits on a layer with fewest zeros is {:?}", result);

    println!("Message after decoding the image:", );
    for row in &image {
        let line:String = row.iter().map(|i| if i == &0 { "██" } else { "  "}).collect();
        println!("{}", line);
    }
}