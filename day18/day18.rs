use std::fs;
use std::u64;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BinaryHeap;

type Coordinates = (i32, i32);
type AreaMap = HashMap<Coordinates, char>;
type Items = Vec<char>;
type ItemLocations = HashMap<char, Coordinates>;
type RouteQueue = VecDeque<(Coordinates, u64, Items)>;
type Routes = HashMap<char, (u64, Items)>;
type RouteMap = HashMap<char, Routes>;
type Processed = HashMap<(Items, u64), u64>;
type KeyMap = HashMap<char, u64>;

const ROW_NUM:[i32; 4] = [-1, 0, 1, 0];
const COL_NUM:[i32; 4] = [ 0, 1, 0,-1];

#[derive(Clone, Eq, PartialEq, Debug)]
struct Element {
    distance: u64,
    key_map: u64,
    items: Items,
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
            .then_with(|| self.key_map.cmp(&other.key_map))
            .then_with(|| self.items.cmp(&other.items))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for Element {
    fn partial_cmp(&self, other: &Element) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn find_best_routes(from:&Coordinates, map:&AreaMap) -> Routes {
    let mut visited:HashSet<Coordinates> = HashSet::new();
    let mut queue = RouteQueue::new();
    let mut routes = Routes::new();

    queue.push_back((*from, 0, Vec::new()));

    while let Some((location, distance, doors)) = queue.pop_front() {
        let mut doors = doors.clone();
        visited.insert(location);

        if distance > 0 {
            let c = map.get(&location);
            match c {
                Some('a'..='z') => {
                    routes.insert(*c.unwrap(), (distance, doors.clone()));
                },
                Some('A'..='Z') => {
                    let mut lower = *c.unwrap();
                    lower.make_ascii_lowercase();
                    doors.push(lower);
                },
                Some(_) => {},
                None => unreachable!(),
            }
        }

        for i in 0..4 {
            let neighbour = (location.0 + COL_NUM[i], location.1 + ROW_NUM[i]);

            if !visited.contains(&neighbour) {
                let c = map.get(&neighbour);

                match c {
                    Some('#') => {},
                    Some(_c) => {
                        queue.push_back((neighbour, distance + 1, doors.clone()));
                    },
                    None => unreachable!(),
                }
            }
        }
    }

    routes
}

fn shortest_distance(robots:&Items, routes:&RouteMap, key_map:&KeyMap) -> u64 {
    let mut priority_queue = BinaryHeap::new();
    let mut processed = Processed::new();

    priority_queue.push(Element { distance: 0, key_map: 0, items: robots.to_vec() });
    processed.insert((robots.to_vec(), 0), 0);

    let full_key_map = !(!0 << key_map.len()); // Set all key bits

    while let Some(Element { distance: current_distance, key_map: current_key_map, items: nodes }) = priority_queue.pop() {
        if current_key_map == full_key_map {
            return current_distance;
        }

        for robot in 0..robots.len() {
            let node = nodes[robot];

            for (next_node, (next_distance, doors_on_path)) in routes.get(&node).unwrap().iter() {
                let new_distance = current_distance + next_distance;
                let next_key_position = key_map.get(next_node).unwrap();

                if !(current_key_map & next_key_position > 0) {
                    let reachable = doors_on_path.iter().all(|door| current_key_map & key_map.get(door).unwrap() > 0);
                    let next_key_map = current_key_map | next_key_position;

                    if reachable {
                        let mut next_nodes = nodes.clone();
                        next_nodes[robot] = *next_node;

                        if !processed.contains_key(&(next_nodes.clone(), next_key_map)) || *processed.get(&(next_nodes.clone(), next_key_map)).unwrap() > new_distance {
                            *processed.entry((next_nodes.clone(), next_key_map)).or_insert(0) = new_distance;
                            priority_queue.push(Element { distance: new_distance, key_map: next_key_map, items: next_nodes.clone()});
                        }
                    }
                }
            }
        }
    }

    0
}

fn main() {
    let data = fs::read_to_string("input.txt").expect("Unable to read file");
    let lines: Vec<_> = data.split('\n').collect();
    let mut map:AreaMap = AreaMap::new();
    let mut keys:ItemLocations = ItemLocations::new();
    let mut doors:ItemLocations = ItemLocations::new();
    let mut items:ItemLocations = ItemLocations::new();
    let mut entrance:Coordinates = (0, 0);

    for (y, line) in lines.iter().enumerate() {
        for (x, c) in line.chars().enumerate() {
            if c == '@' {
                map.insert((x as i32, y as i32), '.');
                entrance = (x as i32, y as i32);
                items.insert(c, (x as i32, y as i32));
            } else {
                map.insert((x as i32, y as i32), c);
            }
            match c {
                'A'..='Z' => {
                    doors.insert(c, (x as i32, y as i32));
                    items.insert(c, (x as i32, y as i32));
                },
                'a'..='z' => {
                    keys.insert(c, (x as i32, y as i32));
                    items.insert(c, (x as i32, y as i32));
                },
                _ => {
                    ();
                }
            };
        }

    }

    let mut key_map = KeyMap::new();
    let mut sorted_keys = keys.keys().map(|&c| c).collect::<Items>();
    sorted_keys.sort();

    for (i, &key) in sorted_keys.iter().enumerate() {
        key_map.insert(key, 1 << i);
    }

    let mut routes = RouteMap::new();

    for (k,v) in keys.iter() {
        routes.insert(*k, find_best_routes(v, &map));
    }

    routes.insert('@', find_best_routes(&entrance, &map));

    // Part 1
    let answer1 = shortest_distance(&vec!['@'], &routes, &key_map);
    println!("Part 1: The shortest path that collects all of the keys is {:?}", answer1);

    // Part 2
    *map.entry(entrance).or_insert('#') = '#';

    for i in 0..4 {
        let pos = (entrance.0 + COL_NUM[i], entrance.1 + ROW_NUM[i]);
        *map.entry(pos).or_insert('#') = '#';
    }

    let robots:[(char, Coordinates); 4] =
        [('1', (entrance.0 - 1, entrance.1 - 1)),
         ('2', (entrance.0 + 1, entrance.1 - 1)),
         ('3', (entrance.0 + 1, entrance.1 + 1)),
         ('4', (entrance.0 - 1, entrance.1 + 1))];


    for (robot, location) in robots.iter() {
        *map.entry(*location).or_insert('#') = *robot;
    }

    routes.clear();

    for (k,v) in keys.iter() {
        routes.insert(*k, find_best_routes(v, &map));
    }

    for (robot, location) in robots.iter() {
        routes.insert(*robot, find_best_routes(location, &map));
    }

    let answer2 = shortest_distance(&vec!['1', '2', '3', '4'], &routes, &key_map);
    println!("Part 2: The fewest steps necessary to collect all of the keys is {:?}", answer2);
}
