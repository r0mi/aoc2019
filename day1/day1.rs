use std::fs;

fn module_fuel_consumption(mass:i32) -> i32 {
    let fuel_amount:i32 = mass / 3 - 2;
    if fuel_amount < 0 {
        0
    } else {
        fuel_amount + module_fuel_consumption(fuel_amount)
    }
}

fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let modules: Vec<i32> = data.split('\n')
                                    .map(|s| s.parse().unwrap())
                                    .collect();

    let mut initial_fuel_amount:i32 = 0;
    let mut total_fuel_amount:i32 = 0;

    for module in modules {
        // println!("Module {} fuel req is {}", module, module_fuel_consumption(module));
        initial_fuel_amount += module / 3 - 2;
        total_fuel_amount += module_fuel_consumption(module);
    }

    println!("The sum of fuel requirements of all the modules is {}", initial_fuel_amount);
    println!("The total sum of fuel requirements of all the modules is {}", total_fuel_amount);
}