use std::{
    collections::{VecDeque, HashMap, BTreeSet},
    sync::{Arc, Mutex},
};

use rayon::prelude::*; // Import Rayon's prelude
use rand::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Cardinal {
    U,
    R,
    D,
    L,
}

macro_rules! lock_as_mut {
    (|$var:ident | $custom_code: block) => {
        let $var = $var.clone();
        if let Ok(mut $var) = $var.lock() {
            $custom_code
        };
    };
}

macro_rules! lock_readonly {
    (|$var:ident | $custom_code: block) => {
        if let Ok($var) = $var.lock() {
            $custom_code
        };
    };
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
            Cardinal::U => {'↑'}, 
            Cardinal::R => {'→'},
            Cardinal::D => {'↓'},
            Cardinal::L => {'←'},
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
    pub fn iter_all() -> Vec<Cardinal> {
        let mut cardinals: Vec<Cardinal> = Vec::new();

        cardinals.push(Cardinal::U);
        cardinals.push(Cardinal::R);
        cardinals.push(Cardinal::D);
        cardinals.push(Cardinal::L);

        return cardinals;
    }

    pub fn inverted_list(&self) -> BTreeSet<Cardinal>{
        let mut cardinals: BTreeSet<Cardinal> = BTreeSet::new();
        cardinals.insert(Cardinal::U);
        cardinals.insert(Cardinal::R);
        cardinals.insert(Cardinal::D);
        cardinals.insert(Cardinal::L);

        cardinals.remove(self);
        cardinals.remove(&self.inverse());
        return cardinals;
    }


}

#[derive(Debug, Clone)]
struct Tile {
    coords: (i32, i32),
    is_obstruction: bool,
    distance_map: HashMap<(i32, i32), usize>,
    cardinal_map: HashMap<(i32, i32), Cardinal>,
}

struct Board {
    tiles: Vec<Tile>,
    size: usize,
}

impl Board {
    pub fn generate_board(size: usize) -> Board {
        let mut tiles: Vec<Tile> = Vec::new();
        let mut rng = rand::thread_rng();
        for j in 0..size {
            for i in 0..size {
                let prob: f64 = rng.gen(); // generates a float between 0 and 1

                if prob > 0.2 {
                    tiles.push(Tile {
                        coords: (i as i32, j as i32),
                        is_obstruction: false,
                        distance_map: HashMap::new(),
                        cardinal_map: HashMap::new(),
                    });
                } else {
                    tiles.push(Tile {
                        coords: (i as i32, j as i32),
                        is_obstruction: true,
                        distance_map: HashMap::new(),
                        cardinal_map: HashMap::new(),
                    });
                }
            }
        }

        Board { tiles, size }
    }

    pub fn get_all_distances(mut board: Board) {

        let tiles = board.tiles.clone();
        let board = Arc::new(Mutex::new(board));
        
        tiles.par_iter().for_each(|tile| {
            let (x, y) = tile.coords;
            let board = board.clone();
            if let Ok(mut board) = board.lock(){
                board.get_cardinals(x, y);
                board.print(tile.coords);
            };
            println!();
        });
    }

    pub fn get_distances(&mut self, x: i32, y: i32) {
        let count:usize = 0;
        let source = (x,y);
        let mut c_stack: VecDeque<((i32, i32),usize)> = VecDeque::new();

        if let Some(tile) = self.get_mut_tile(x, y){
            tile.distance_map.insert(source, count);
        }
        for _ in 0..self.get_next_neighbor(source, x, y).1 {
            c_stack.push_back(((x,y), count));
        }

        loop {
            if let Some(((nx, ny), count)) = c_stack.pop_back() {
                if let Some(cardinal) = self.get_next_neighbor(source, nx, ny).0{
                    c_stack = self.traverse_cardinal(source, c_stack, cardinal, nx, ny, count);
                }
            } else {
                break;
            }
        }
    }

    pub fn get_cardinals(&mut self, x: i32, y: i32) {
        
        let source = (x,y);
        self.get_distances(x, y);
        for j in 0..(self.size) as i32 {
            for i in 0..(self.size) as i32 {
                let mut cardinal: Option<Cardinal> = None;
                if let Some(tile) = self.get_mut_tile(i,j){
                    if !tile.is_obstruction{
                        cardinal = self.get_shortest_cardinal(source, i, j);
                    }
                }
                if let Some(tile) = self.get_mut_tile(i,j){
                    if let Some(cardinal) = cardinal{
                        tile.cardinal_map.insert(source, cardinal);
                    }
                }
            }
        }
    }
    pub fn get_shortest_cardinal_dest(&mut self, source:(i32,i32), current_tile: (i32,i32)) -> Option<Cardinal>{
        let mut shortest_cardinal = None;
        let mut shortest: usize = usize::MAX;
        for cardinal in Cardinal::iter_all(){
            let (dx, dy) = cardinal.delta();
            
            if let Some(tile) = self.get_tile(current_tile.0+dx, current_tile.1+dy){
                if let Some(distance) = tile.distance_map.get(&source){
                    if distance < &shortest {
                        shortest = *distance;
                        shortest_cardinal = Some(cardinal);
                    }
                }
            }
        }

        return shortest_cardinal;
    }

    pub fn get_shortest_cardinal(&self, source:(i32,i32), x: i32, y: i32) -> Option<Cardinal>{
        let mut shortest_cardinal = None;
        let mut shortest: usize = usize::MAX;
        for cardinal in Cardinal::iter_all(){
            let (dx, dy) = cardinal.delta();
            
            if let Some(tile) = self.get_tile(x+dx, y+dy){
                if let Some(distance) = tile.distance_map.get(&source){
                    if distance < &shortest {
                        shortest = *distance;
                        shortest_cardinal = Some(cardinal);
                    }
                }
            }
        }

        return shortest_cardinal;
    }

    pub fn traverse_cardinal(
        &mut self,
        source: (i32, i32),
        mut c_stack: VecDeque<((i32, i32), usize)>,
        cardinal: Cardinal,
        mut x: i32,
        mut y: i32,
        mut count: usize
    ) -> VecDeque<((i32, i32), usize)> {
        let (dx, dy) = cardinal.delta();

        loop{
            count += 1;
            x += dx;
            y += dy;
            if x<0 || y <0 || x>=self.size as i32 || y >=self.size as i32{
                break;
            }


            let has_neighbors = self.get_next_neighbor(source, x, y).1;

            if let Some(tile) = self.get_mut_tile(x,y) {
                if tile.is_obstruction || tile.distance_map.contains_key(&source){
                    break;
                }
                tile.distance_map.insert(source, count);
                for _ in 0..has_neighbors {
                    c_stack.push_front(((x,y), count));
                }
            } else {
                break;
            }
        }
        c_stack
    }

    fn get_next_neighbor(&mut self,source: (i32,i32) , x: i32, y: i32,) -> (Option<Cardinal>, usize) {
        let mut cardinal_opt: Option<Cardinal> = None;
        let mut count:usize = 0;
        for cardinal in Cardinal::iter_all() {
            let (dx, dy) = cardinal.delta(); 

            if let Some(tile) = self.get_tile(x+dx, y+dy){
                if !(tile.distance_map.contains_key(&source) || tile.is_obstruction) {
                    cardinal_opt = Some(cardinal);
                    count += 1;
                }
            }
        }
        return (cardinal_opt, count);
    }


    fn get_vec_idx(&self, x: i32, y: i32) -> Option<usize> {
        if x>=0 && y>=0 && x<self.size as i32 && y < self.size as i32 {
            Some(((y * self.size as i32) + x) as usize)
        } else {
            None
        }
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Option<&Tile> {
        if let Some(pos) = self.get_vec_idx(x, y){
            return self.tiles.get(pos);
        } else {
            return None;
        }
    }

    pub fn get_mut_tile(&mut self, x: i32, y: i32) -> Option<&mut Tile> {
        if let Some(pos) = self.get_vec_idx(x, y){
            return self.tiles.get_mut(pos);
        } else {
            return None;
        }
    }

    pub fn print_dis(&self, source: (i32,i32)) {
        let mut to_print = String::new();
        for j in 0..self.size {
            for i in 0..self.size {
                if let Some(tile) = self.get_tile(i as i32,j as i32) {
                    if source == (i as i32,j as i32) {
                        to_print.push_str("  0");
                    } else if let Some(count) = tile.distance_map.get(&source){
                        if count < &10 {
                            let count = count.to_string();
                            to_print.push_str(&format!("  {}", count));
                        } else if count< &100{
                            let count = count.to_string();
                            to_print.push_str(&format!(" {}", count));
                        } else {
                            to_print.push_str("  #");
                        }
                    } else if tile.is_obstruction {
                        to_print.push_str("  #");
                    } else {
                        to_print.push_str("  .");
                    }
                }
            }
            to_print.push('\n');
        }
        println!("{}", to_print);
    }

    pub fn print(&self, source: (i32,i32)) {
        let mut to_print = String::new();
        for j in 0..self.size {
            for i in 0..self.size {

                if let Some(tile) = self.get_tile(i as i32, j as i32) {
                    if source == (i as i32,j as i32) {
                        to_print.push('0');
                    } else if tile.is_obstruction {
                        to_print.push('#');
                    } else if let Some(cardinal) = tile.cardinal_map.get(&source){
                        to_print.push(cardinal.get_char_eq());
                    } else {
                        to_print.push('.');
                    }
                }
            }
            to_print.push('\n');
        }
        println!("{}", to_print);
    }
}

fn main() {


}
#[cfg(test)]
mod tests { // TODO: Make tests better and add assertions. Also, probably will want to move these somewhere else at some point

    use super::*;

    #[test]
    fn performance_run(){
        //current runtimes 20:1s, 50:5.5s, 60:12s
        //not looking good rn
        
        let size: usize = 20;
        let board = Board::generate_board(size);
        Board::get_all_distances(board);
        let size: usize = 50;
        let board = Board::generate_board(size);
        Board::get_all_distances(board);
        let size: usize = 60;
        let board = Board::generate_board(size);
        Board::get_all_distances(board);
    }

}


