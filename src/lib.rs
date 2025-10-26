use wasm_bindgen::prelude::*;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Cardinal {
    U,
    R,
    D,
    L,
}

impl Cardinal {
    pub fn inverse(&self) -> Cardinal {
        match self {
            Cardinal::U => Cardinal::D,
            Cardinal::R => Cardinal::L,
            Cardinal::D => Cardinal::U,
            Cardinal::L => Cardinal::R,
        }
    }
    pub fn get_char_eq(&self) -> char {
        match self {
            Cardinal::U => '↑',
            Cardinal::R => '→',
            Cardinal::D => '↓',
            Cardinal::L => '←',
        }
    }

    pub fn delta(&self) -> (i32, i32) {
        match self {
            Cardinal::U => (0, -1),
            Cardinal::R => (1, 0),
            Cardinal::D => (0, 1),
            Cardinal::L => (-1, 0),
        }
    }
    pub fn delta_from(&self, coord: (i32, i32)) -> (i32, i32) {
        let (dx, dy) = self.delta();
        (coord.0 + dx, coord.1 + dy)
    }

    pub fn iter_all() -> Vec<Cardinal> {
        let mut cardinals: Vec<Cardinal> = Vec::new();

        cardinals.push(Cardinal::U);
        cardinals.push(Cardinal::R);
        cardinals.push(Cardinal::D);
        cardinals.push(Cardinal::L);

        return cardinals;
    }

    pub fn to_string(&self) -> String{
        match self {
            Cardinal::U => String::from("U"),
            Cardinal::R => String::from("R"),
            Cardinal::D => String::from("D"),
            Cardinal::L => String::from("L"),
        }
    }
}

fn get_distances(
    source: (i32, i32),
    tiles: &HashSet<(i32, i32)>,
) -> HashMap<(i32, i32), (usize, Cardinal)> {
    let mut distances: HashMap<(i32, i32), (usize, Cardinal)> = HashMap::new();
    let mut c_stack: VecDeque<((i32, i32), (Cardinal, usize))> = VecDeque::new();
    for neighbor in Cardinal::iter_all() {
        if tiles.contains(&neighbor.delta_from(source)) {
            c_stack.push_front((source, (neighbor, 0)));
        }
    }

    loop {
        //print_tiles(40, tiles, &distances);
        if let Some((coord, (cardinal, count))) = c_stack.pop_back() {
            (distances, c_stack) = traverse(coord, count, cardinal, &tiles, c_stack, distances);
        } else {
            break;
        }
    }

    distances
}

fn traverse(
    source: (i32, i32),
    mut count: usize,
    cardinal: Cardinal,
    tiles: &HashSet<(i32, i32)>,
    mut c_stack: VecDeque<((i32, i32), (Cardinal, usize))>,
    mut distances: HashMap<(i32, i32), (usize, Cardinal)>,
) -> (
    HashMap<(i32, i32), (usize, Cardinal)>,
    VecDeque<((i32, i32), (Cardinal, usize))>,
) {
    let mut current_coord = source;

    loop {
        current_coord = cardinal.delta_from(current_coord);
        count += 1;
        if !tiles.contains(&current_coord) || distances.contains_key(&current_coord) {
            break;
        } else {
            distances.insert(current_coord, (count, cardinal.inverse()));
        }
        let mut min = usize::MAX;
        for neighbor in Cardinal::iter_all() {
            let delta = neighbor.delta_from(current_coord);
            if let Some((size, _stored_cardinal)) = distances.get_mut(&delta) {
                if *size < min {
                    min = *size;
                    if let Some((_current_size, current_coord)) = distances.get_mut(&current_coord)
                    {
                        *current_coord = neighbor;
                    }
                }
            } else {
                if neighbor != cardinal {
                    c_stack.push_front((current_coord, (neighbor, count)));
                }
            }
        }
    }

    (distances, c_stack)
}

fn print_tiles(
    source: (i32, i32),
    size: i32,
    tiles: &HashSet<(i32, i32)>,
    distances: &HashMap<(i32, i32), (usize, Cardinal)>,
) -> String {
    let mut to_print = String::new();
    for j in 0..size {
        for i in 0..size {
            let coords = (i as i32, j as i32);
            if coords == source {
                to_print.push('0');
            } else if let Some((_distance, cardinal)) = distances.get(&coords) {
                to_print.push(cardinal.get_char_eq());
            } else if tiles.contains(&coords) {
                to_print.push('.');
            } else {
                to_print.push('#');
            }
        }
        to_print.push('\n');
    }
    format!("{}", to_print)
}

fn print_tiles_dis(
    source: (i32, i32),
    size: i32,
    tiles: &HashSet<(i32, i32)>,
    distances: &HashMap<(i32, i32), (usize, Cardinal)>,
) -> String {
    let mut to_print = String::new();
    for j in 0..size {
        for i in 0..size {
            let coords = (i as i32, j as i32);
            if coords == source {
                to_print.push_str("  0");
            } else if let Some((distance, _cardinal)) = distances.get(&coords) {
                if *distance < 10 as usize {
                    to_print.push_str(&format!("  {}", distance));
                } else {
                    to_print.push_str(&format!(" {}", distance));
                }
            } else if tiles.contains(&coords) {
                to_print.push_str("  .");
            } else {
                to_print.push_str("  #");
            }
        }
        to_print.push('\n');
    }
    format!("{}", to_print)
}

fn cord_to_str(cord: &(i32, i32)) -> String {
    format!("{}:{}", cord.0, cord.1)
}

#[wasm_bindgen]
pub fn find_paths(world: Option<String>) -> String {
    match world {
        Some(world) => {
            let world: Vec<Vec<i8>> = serde_json::from_str(world.as_str()).unwrap();
            let size = world.len() as i32;
            let mut cord_set: HashSet<(i32, i32)> = HashSet::new();

            for (i, row) in world.into_iter().enumerate() {
                let i = i as i32;
                for (j, element) in row.into_iter().enumerate() {
                    let j = j as i32;
                    if element == 0 {
                        cord_set.insert((j,i));                    
                    }
                }
            }


            let distance_maps: HashMap<String, HashMap<String, String>> = cord_set
                .par_iter()
                .map(|tile| {
                    let distances = get_distances(*tile, &cord_set);
                    let cord_str = cord_to_str(tile);
                    let mut easy_map: HashMap<String, String> = HashMap::new();
                    for (key, (dis, cardinal)) in distances {
                        let key_cord_str = cord_to_str(&key);
                        easy_map.insert(key_cord_str, cardinal.to_string());
                    }
                    (cord_str, easy_map)
                })
                .collect();
            

            serde_json::to_string(&distance_maps).unwrap_or(format!("oof"))
        },
        None => {
            format!("oof")
        }
    }
}

