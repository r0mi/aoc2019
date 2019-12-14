use std::collections::HashMap;

type Reaction = HashMap<String, (i64, Vec<(i64, String)>)>;
type Leftovers = HashMap<String, i64>;

fn parse_chemical(input_str:&str) -> (i64, String) {
    let mut iter = input_str.trim().split(" ");
    let quantity = iter.next().unwrap().parse().unwrap();
    let chemical = iter.next().unwrap().to_string();
    (quantity, chemical)
}

fn calc_ore_required(chemical: &str, amount: i64, reactions: &Reaction, leftovers: &mut Leftovers) -> i64 {
    let (produced_qty, ingredients) = reactions.get(&chemical.to_string()).unwrap();
    let available = leftovers.entry(chemical.to_string()).or_insert(0);
    let needed = 0.max(amount - *available);
    let multiple = (needed as f64 / *produced_qty as f64).ceil() as i64;
    let leftover = multiple * produced_qty - amount;

    *available += leftover;

    let mut ore_required = 0;

    for ingredient in ingredients {
        if ingredient.1 == "ORE" {
            ore_required += ingredient.0 * multiple;
        } else {
            ore_required += calc_ore_required(&ingredient.1, ingredient.0 * multiple, reactions, leftovers);
        }
    }

    ore_required
}

fn main() {
    let lines = include_str!("input.txt").trim_end().lines().collect::<Vec<_>>();
    let mut reactions = Reaction::new();

    for reaction in lines {
        let mut reaction_parts = reaction.split(" => ");
        let inputs:Vec<(i64, String)> = reaction_parts.next().unwrap().split(",").map(|s| parse_chemical(s)).collect::<Vec<_>>();
        let output = parse_chemical(reaction_parts.next().unwrap());
        reactions.insert(output.1, (output.0, inputs));
    }

    let ore_for_1_fuel = calc_ore_required("FUEL", 1, &reactions, &mut Leftovers::new());

    println!("The minimum amount of ORE required to produce exactly 1 FUEL is {:?}", ore_for_1_fuel);

    let ore_stock = 1000000000000;
    let mut lower_bound = ore_stock / ore_for_1_fuel;
    let mut upper_bound = lower_bound * 2;
    let mut max_fuel = 0;

    while lower_bound < upper_bound {
        max_fuel = (lower_bound + upper_bound) / 2;
        let consumed_ore = calc_ore_required("FUEL", max_fuel, &reactions, &mut Leftovers::new());
        if consumed_ore > ore_stock {
            upper_bound = max_fuel - 1;
        } else {
            lower_bound = max_fuel + 1;
        }
    }

    println!("From 1 trillion ORE you can produce {:?} FUEL", max_fuel);
}