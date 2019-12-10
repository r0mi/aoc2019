use std::collections::HashMap;
use std::collections::HashSet;
use std::f64;
use std::cmp::Eq;
use std::hash::Hash;
use std::hash::Hasher;
use std::cmp::PartialOrd;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct Float(f64);

impl Float {
    fn canonicalize(&self) -> i64 {
        (self.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0).round() as i64
    }
}

impl PartialEq for Float {
    fn eq(&self, other: &Float) -> bool {
        self.canonicalize() == other.canonicalize()
    }
}

impl Eq for Float {}

impl Hash for Float {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.canonicalize().hash(state);
    }
}

impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Float) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Float {
    fn cmp(&self, other: &Float) -> Ordering {
        self.canonicalize().cmp(&other.canonicalize())
    }
}

fn gcd(a:i32, b:i32) -> i32 {
    if b == 0 {
        return a
    } else {
        return gcd(b, a % b)
    }
}

fn is_blocked(x1:&i32, y1:&i32, x2:&i32, y2:&i32, map:&HashSet<(i32, i32)>) -> bool {
    let diff_x = (x2 - x1).abs();
    let diff_y = (y2 - y1).abs();

    if diff_x + diff_y == 1 || diff_x == 1 && diff_y == 1 {
        return false
    } else {
        let div = gcd(diff_x, diff_y);
        let step_x:i32;
        let step_y:i32;

        if x2 >= x1 {
            step_x = diff_x / div;
        } else {
            step_x = diff_x / -div;
        }

        if y2 >= y1 {
            step_y = diff_y / div;
        } else {
            step_y = diff_y / -div;
        }

        for i in 1..div {
            if map.contains(&(x1 + step_x * i, y1 + step_y * i)) {
                return true
            }
        }
    }
    return false
}

fn main() {
    let lines = include_str!("input.txt").trim_right().lines().collect::<Vec<_>>();
    let mut asteroid_set:HashSet<(i32, i32)> = HashSet::new();

    for (y, row) in lines.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            if c == '#' {
                asteroid_set.insert((x as i32, y as i32));
            }
        }
    }

    let mut max_visible_asteroids:i32 = 0;
    let (mut best_location_x, mut best_location_y) = (0, 0);

    for (x1, y1) in &asteroid_set {
        let mut visible_asteroids:i32 = 0;

        for (x2, y2) in &asteroid_set {
            if !(x1 == x2 && y1 == y2) {
                if !is_blocked(x1, y1, x2, y2, &asteroid_set) {
                    visible_asteroids += 1;
                }
            }
        }

        if visible_asteroids > max_visible_asteroids {
            max_visible_asteroids = visible_asteroids;
            best_location_x = *x1;
            best_location_y = *y1;
        }
    }

    println!("Best asteroid location is {:?},{:?} with visibility of {} other asteroids", best_location_x, best_location_y, max_visible_asteroids);

    let mut asteroid_distances_at_angle:HashMap<Float, Vec<Float>> = HashMap::new();
    let mut angle_distance_asteroid_map:HashMap<(Float, Float), (i32, i32)> = HashMap::new();
    let mut angles:Vec<Float> = Vec::new();

    asteroid_set.remove(&(best_location_x, best_location_y));

    for (x, y) in &asteroid_set {
        let delta_x:f64;
        let delta_y:f64;
        let angle:f64;
        if x < &best_location_x && y <= &best_location_y { // 4. quadrant
            delta_x = (best_location_x - x) as f64;
            delta_y = (best_location_y - y) as f64;
            angle = f64::consts::PI + f64::consts::FRAC_PI_2 + (delta_y / delta_x).atan();
        } else if x >= &best_location_x && y < &best_location_y { // 1. quadrant
            delta_x = (x - best_location_x) as f64;
            delta_y = (best_location_y - y) as f64;
            angle = (delta_x / delta_y).atan();
        } else if x > &best_location_x && y >= &best_location_y { // 2. quadrant
            delta_x = (x - best_location_x) as f64;
            delta_y = (y - best_location_y) as f64;
            angle = f64::consts::FRAC_PI_2 + (delta_y / delta_x).atan();
        } else { // 3. quadrant
            delta_x = (best_location_x - x) as f64;
            delta_y = (y - best_location_y) as f64;
            angle = f64::consts::PI + (delta_x / delta_y).atan();
        }
        let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();
        let mut v = asteroid_distances_at_angle.entry(Float(angle)).or_insert(vec![]);
        v.push(Float(distance));
        v.sort();
        v.reverse();
        angles.push(Float(angle));
        angle_distance_asteroid_map.insert((Float(angle), Float(distance)), (*x, *y));
    }

    angles.sort();
    angles.dedup();

    let mut vaporized = 0;

    while vaporized < asteroid_set.len() {
        for angle in &angles {
            if let Some(distances) = asteroid_distances_at_angle.get_mut(angle) {
                if let Some(distance) = distances.pop() {
                    if let Some((x, y)) = angle_distance_asteroid_map.get(&(angle.clone(), distance)) {
                        vaporized += 1;

                        if vaporized == 200 {
                            println!("The 200th asteroid to be vaporized is at {:?},{:?} = {:?}", x, y, x * 100 + y);
                            return;
                        }
                        // println!("The {}. asteroid to be vaporized is at {:?},{:?}", vaporized, x, y);
                    }
                }
            }
        }
    }
}