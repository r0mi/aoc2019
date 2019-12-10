use std::fs;
use std::collections::HashMap;
use std::collections::HashSet;

fn calc_orbit_length(object:&String, orbits:&HashMap<String, String>) -> u32 {
    match orbits.get(object) {
        Some(obj) => {
            if obj == "COM" {
                1
            } else {
                1 + calc_orbit_length(obj, orbits)
            }
        },
        None => panic!("No such object"),
    }
}

fn map_orbit_path(object:&String, orbits:&HashMap<String, String>, path: &mut HashSet<String>) {
    match orbits.get(object) {
        Some(obj) => {
            if obj == "COM" {
                return
            } else {
                path.insert(obj.clone());
                map_orbit_path(obj, orbits, path)
            }
        },
        None => panic!("No such object"),
    }
}

fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let orbits: Vec<String> = data.split('\n')
                                    .map(|s| s.to_string())
                                    .collect();
    let mut object_set:HashSet<String> = HashSet::new();
    let mut object_map:HashMap<String, String> = HashMap::new();

    for orbit in orbits {
        let mut objects = orbit.split(')').map(|s| s.to_string()).collect::<Vec<String>>();
        let secondary = objects.pop().unwrap();
        let main = objects.pop().unwrap();
        if main != "COM" {
            object_set.insert(main.clone());
        }
        object_set.insert(secondary.clone());
        object_map.insert(secondary, main);
    }

    let mut orbit_count = 0;

    for ref mut obj in &object_set {
        orbit_count += calc_orbit_length(&obj, &object_map);
    }

    println!("Total number of direct and indirect orbits is {:?}", orbit_count);

    let mut you_set:HashSet<String> = HashSet::new();
    map_orbit_path(&("YOU".to_string()), &object_map, &mut you_set);

    let mut san_set:HashSet<String> = HashSet::new();
    map_orbit_path(&("SAN".to_string()), &object_map, &mut san_set);

    let you_set_clone = you_set.clone();
    let san_set_clone = san_set.clone();

    let intersection:HashSet<_> = san_set_clone.intersection(&you_set_clone).collect();
    let union:HashSet<_> = san_set.union(&you_set).collect();
    let diff:HashSet<_> = union.difference(&intersection).collect();

    println!("Minimum number of orbital transfers required is {:?}", diff.len());

}