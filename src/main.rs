
use std::fs::File;
use std::io::{self, BufRead};
use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

const WIDTH : i32 = 9;
const HEIGHT : i32 = 6;
const COST_COEF : f32 = 1.0;
const EUCLID_COEF : f32 = 1.0;

#[derive(Debug, Clone, PartialEq)]
struct Direction {
    w: bool,
    e : bool,
    n : bool,
    s : bool,

}

#[derive(Debug,Clone)]
struct Node {
    allowed_direction : Direction,
    door_direction : Direction,
    is_key : bool,
    is_end : bool,
    ord: i32
}

#[derive(Debug, Clone, PartialEq)]
struct NodeN {
    allowed_direction: Direction,
    door_direction : Direction,
    mark_i : usize,
    mark_j : usize,
    is_key: bool,
    is_end : bool
}

impl NodeN{
    fn valid_coords(&self, mark_i : i32, mark_j : i32) -> bool {
        if mark_i <0 || mark_j < 0 || mark_i > WIDTH || mark_j > HEIGHT {
            return false
        }

        //todo : ubaciti to da ne sme kroz zidove da prodje
        true
    }
    fn valid_next_node(&self) -> bool {
        if self.mark_i <0 || self.mark_j < 0 || self.mark_i > WIDTH as usize -1 || self.mark_j > HEIGHT as usize -1 {
            return false
        }
        true
    }

    fn get_idx(&self) -> usize {
        self.mark_j*9+self.mark_j
    }

    fn get_linked_nodes(&self, keys : i32, pure_search : bool) -> Vec<NodeN> {
        let mut neighbours : Vec<NodeN> = Vec::new();
        let nodes = get_nodes_n(get_nodes());

        let index = self.mark_j*9+self.mark_i;
        let node = nodes[index].clone();
        if pure_search{
            if node.allowed_direction.w  && node.valid_coords(self.mark_i as i32 - 1, self.mark_j as i32) && valid_coordinates(self.mark_i as i32 - 1, self.mark_j as i32) {
                neighbours.push(nodes[index-1].clone());
            }
            if node.allowed_direction.e  && node.valid_coords(self.mark_i as i32 + 1, self.mark_j as i32) && valid_coordinates(self.mark_i as i32 + 1, self.mark_j as i32) {
                neighbours.push(nodes[index+1].clone());
            }
            if node.allowed_direction.n && node.valid_coords(self.mark_i as i32, self.mark_j as i32 - 1) && valid_coordinates(self.mark_i as i32 , self.mark_j as i32 -1) {
                neighbours.push(nodes[index-9].clone());
            }
            if node.allowed_direction.s &&  node.valid_coords(self.mark_i as i32, self.mark_j as i32 + 1) && valid_coordinates(self.mark_i as i32, self.mark_j as i32 + 1) {
                neighbours.push(nodes[index+9].clone());
            }
        }else{
            if node.allowed_direction.w  && node.valid_coords(self.mark_i as i32 - 1, self.mark_j as i32) && valid_coordinates(self.mark_i as i32 - 1, self.mark_j as i32)  && (!node.door_direction.w || (node.door_direction.w && keys > 0)){
                neighbours.push(nodes[index-1].clone());
            }
            if node.allowed_direction.e  && node.valid_coords(self.mark_i as i32 + 1, self.mark_j as i32) && valid_coordinates(self.mark_i as i32 + 1, self.mark_j as i32) && (!node.door_direction.e || (node.door_direction.e && keys > 0)){
                neighbours.push(nodes[index+1].clone());
            }
            if node.allowed_direction.n && node.valid_coords(self.mark_i as i32, self.mark_j as i32 - 1) && valid_coordinates(self.mark_i as i32 , self.mark_j as i32 -1) && (!node.door_direction.n || (node.door_direction.n && keys > 0)){
                neighbours.push(nodes[index-9].clone());
            }
            if node.allowed_direction.s &&  node.valid_coords(self.mark_i as i32, self.mark_j as i32 + 1) && valid_coordinates(self.mark_i as i32 , self.mark_j as i32 + 1)  && (!node.door_direction.s || (node.door_direction.s && keys > 0)){
                neighbours.push(nodes[index+9].clone());
            }
        }

        neighbours
    }
}

fn valid_coordinates(i : i32 , j : i32 ) -> bool {
    let ind = j * WIDTH + i;
    return ind < WIDTH * HEIGHT && ind >= 0
}

#[derive(Debug, Clone, PartialEq)]
struct State {
    parent: Box<Option<State>>,
    node : NodeN,
    cost: i32,
    level: i32,
    keys: i32
    // walked_trough_door: bool
}

impl State
{
    fn decide_if_went_trough_door(&self, diff : i32) -> bool {
        match diff {
            -1 => { self.node.door_direction.e },
             1 => { self.node.door_direction.w },
             9 => { self.node.door_direction.n },
            -9 => { self.node.door_direction.s },
            _ => { false }
        }
    }

    fn next_state(&self, node:NodeN) -> State{
        let current_state_index = self.node.mark_j * WIDTH as usize + self.node.mark_i;
        let new_state_index = node.mark_j * WIDTH as usize + node.mark_i;
        // println!("prelazim iz {} u {}", current_state_index, new_state_index);

        let mut keys = self.keys;
        if node.is_key {

            keys += 1;
        }

        let diff = current_state_index as i32 - new_state_index as i32;

        if self.decide_if_went_trough_door(diff){
            keys -= 1;
        }

        State{
            node,
            parent: Box::from(Option::Some(self.clone())),
            cost : self.cost + 1,
            level : self.level + 1,
            keys
        }
    }

    fn is_end_state(&self) -> bool{
        self.node.is_end
    }

    fn possible_next_state(&self, pure_search : bool) -> Vec<State> {
        let mut neighbours : Vec<State> = Vec::new();
        for node in self.node.get_linked_nodes(self.keys, pure_search) {
            let st = self.next_state(node);

            neighbours.push(st);
        };

        neighbours
    }
    fn is_circular_path(&self) -> bool {
        let mut tt = self.parent.clone();
        while tt!= Box::new(Option::None) {
            let n = tt.clone().unwrap().node;
            if n.mark_j * WIDTH as usize + n.mark_i == self.node.mark_i+self.node.mark_j * WIDTH as usize{
                return true
            }
            tt = tt.clone().unwrap().parent;
        }
        false
    }

    fn path(&self) -> Vec<i32>{
        let mut path : Vec<i32> = Vec::new();
        let mut tt = Box::new(Option::Some(self.clone()));
        while tt !=Box::new(Option::None){
            let a = tt.clone().unwrap();
            let idx = a.node.mark_j * WIDTH as usize + a.node.mark_i;
            // println!("{:?} {}", a.clone().node, a.clone().keys);
            path.insert(0, idx as i32);
            tt = tt.unwrap().parent.clone();
        }

         path
    }
}


fn transform_key_or_end(part : &[u8]) -> bool {
    part[0] == 49 && part[1] == 49
}

fn transform_direction(direction: &[u8]) -> Direction {
    let mut dir = Direction{
        w: false,
        e: false,
        n: false,
        s: false
    };

    let w = direction[0];
    if w == 49 {
        dir.w = true;
    }
    let e = direction[1];
    if e == 49 {
        dir.e = true;
    }
    let n = direction[2];
    if n == 49 {
        dir.n = true;
    }
    let s = direction[3];
    if s == 49 {
        dir.s = true;
    }
    dir
}

fn transform_line(line : &String, ord : i32) -> Node{
    let splitted : Vec<&str> = line.trim().split(' ').collect();

    let allowed_direction = transform_direction(splitted[0].as_bytes());
    let door_direction = transform_direction(splitted[1].as_bytes());
    let is_key = transform_key_or_end(&splitted[2][0..2].as_bytes());
    let is_end = transform_key_or_end(&splitted[2][2..].as_bytes());

    Node{
        door_direction,
        allowed_direction,
        is_end,
        is_key,
        ord
    }

}

fn dfs(start : &State) -> Option<State>{
    let mut states_to_be_processed : Vec<State> = Vec::new();
    states_to_be_processed.push(start.clone());
    while states_to_be_processed.len() > 0 {
        let processed_state = states_to_be_processed[0].clone();

        if !processed_state.is_circular_path(){
            // println!("AAAAA j: {}, i: {}, idx {}, keys {}",processed_state.node.mark_j, processed_state.node.mark_i, processed_state.node.mark_i + WIDTH as usize * processed_state.node.mark_j, processed_state.keys );
            if processed_state.is_end_state() {
                return Option::Some(processed_state);
            }
            let mut possible_states = processed_state.possible_next_state(true);
            for s in possible_states{
                states_to_be_processed.insert(0,s);
            }
        }
        states_to_be_processed.retain(|x| x.node.mark_i != processed_state.node.mark_i || x.node.mark_j != processed_state.node.mark_j);

    }

    Option::None
}

fn heuristic_function(state : &State, end : &NodeN) -> f32 {
    // println!("{:?}", end.clone());
    EUCLID_COEF * ((state.node.mark_i as i32 - end.mark_i as i32).pow(2) as f32 +  (state.node.mark_j as i32 - end.mark_j as i32).pow(2) as f32).sqrt() + COST_COEF* state.cost as f32
}

fn a_star(start : &State, end_node : &NodeN, pure_search : bool) -> Option<State>{
    let mut states_to_be_processed : Vec<State> = Vec::new();
    states_to_be_processed.push(start.clone());

    while states_to_be_processed.len() > 0 {
        let processed_state = get_best_state(&states_to_be_processed, end_node);

        if !processed_state.is_circular_path(){
            // println!("AAAAA j: {}, i: {}, idx {}, keys {}",processed_state.node.mark_j, processed_state.node.mark_i, processed_state.node.mark_i + WIDTH as usize * processed_state.node.mark_j, processed_state.keys );
            if processed_state.is_end_state() && processed_state.clone().node.mark_i == end_node.clone().mark_i && processed_state.clone().node.mark_j == end_node.clone().mark_j  {
                return Option::Some(processed_state);
            }
            let mut possible_states = processed_state.possible_next_state(pure_search);
            for s in possible_states{
                states_to_be_processed.insert(0,s);
            }
        }
        states_to_be_processed.retain(|x| x.node.mark_i != processed_state.node.mark_i || x.node.mark_j != processed_state.node.mark_j);

    }

    Option::None
}


fn a_depth(start : &State, end_node : &NodeN, keys : &mut Vec<NodeN>, num_of_doors : i32) -> Vec<i32>{

    let mut keys_left = num_of_doors;
    let mut result : Option<State> = Option::Some(start.clone());
    let mut result1 : State = start.clone();

    let mut paths : Vec<i32> = Vec::new();
    let mut path : Vec<i32> = Vec::new();

    while keys_left > 0 {
        // println!("keys left {}", keys_left.clone());
        result = collect_key(&result1.clone(),end_node,keys);
        match result.clone() {
            Some(mut s) => {
                keys_left -=1;
                path = s.path();
            },
            None => {
                println!("Couldnt collect key");
            }
        }

        path.remove(path.len() -1 );

        for (i, element) in path.iter().enumerate()  {
                paths.insert(0,element.clone());
        }



        result1 = result.clone().unwrap();
        result1.parent = Box::new(Option::None);
    }
    let end_result = a_star(&result1, end_node,false);
    match end_result {
        Some(s) => {
            path = s.path();
        },
        None => {
            println!("some kind of error occured");
        }
    }
    for i in path {
        paths.insert(0,i);
    }

    paths.reverse();

    paths
}

fn collect_key(start : &State, end_node : &NodeN, keys: &mut Vec<NodeN>) -> Option<State> {
    let mut states_to_be_processed : Vec<State> = Vec::new();
    states_to_be_processed.push(start.clone());

    while states_to_be_processed.len() > 0 {
        let processed_state = get_best_state(&states_to_be_processed, end_node);

        if !processed_state.is_circular_path(){
            if keys.contains(&processed_state.node){
                keys.retain(|x| x.mark_i!=processed_state.clone().node.mark_i && x.mark_j != processed_state.clone().node.mark_j);
                return Option::Some(processed_state);

            }

            let mut possible_states = processed_state.possible_next_state(false);
            for s in possible_states{
                states_to_be_processed.insert(0,s);
            }
        }
        states_to_be_processed.retain(|x| x.node.mark_i != processed_state.node.mark_i || x.node.mark_j != processed_state.node.mark_j);

    }

    Option::None
}
fn get_best_state( states : &Vec<State>, end : &NodeN) -> State {
    let mut minimal_heuristic = f32::MAX;
    let mut best_state : Option<&State> = Option::None;
    for s in states {
        let heur = heuristic_function(s, end);
        if heur < minimal_heuristic {
            minimal_heuristic = heur;
            best_state = Option::Some(s);

        }
    }

    best_state.unwrap().clone()
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn get_nodes() -> Vec<Node> {
    let mut nodes: Vec<Node>  = Vec::new();
    let mut i = 0;
    if let Ok(lines) = read_lines("./src/lybrinth.bin") {
        for line in lines {
            if let Ok(line) = line {
                nodes.push(transform_line(&line, i));
                i +=1;
            }
        }
    }
    nodes
}

fn get_nodes_n(nodes : Vec<Node>) -> Vec<NodeN> {
    let mut nodes_n: Vec<NodeN> = Vec::new();
    for j in 0..HEIGHT {
        for i in 0..WIDTH{
            let ind = j * WIDTH + i;
            let node = nodes[ind as usize].clone();
            let node_n = NodeN{
                is_end:node.is_end,
                is_key:node.is_key,
                allowed_direction: node.allowed_direction,
                door_direction:node.door_direction,
                mark_i:i as usize,
                mark_j:j as usize,
            };
            nodes_n.push(node_n);
        }
    }

    nodes_n
}

fn get_nodes_n_with_keys(nodes : &Vec<Node>) -> Vec<NodeN> {
    let mut nodes_n: Vec<NodeN> = Vec::new();
    for j in 0..HEIGHT {
        for i in 0..WIDTH{
            let ind = j * WIDTH + i;
            let node = nodes[ind as usize].clone();
            if node.is_key{
                let node_n = NodeN{
                    is_end:node.is_end,
                    is_key:node.is_key,
                    allowed_direction: node.allowed_direction,
                    door_direction:node.door_direction,
                    mark_i:i as usize,
                    mark_j:j as usize,
                };
                nodes_n.push(node_n);
            }

        }
    }

    nodes_n
}

fn get_end(nodes : &Vec<NodeN>) -> Vec<NodeN> {
    let mut ends : Vec<NodeN> = Vec::new();
    for j in 0..HEIGHT {
        for i in 0..WIDTH{
            let ind = j * WIDTH + i;
            let node = nodes[ind as usize].clone();
            if node.is_end{
                ends.push(NodeN{
                    is_end:node.is_end,
                    is_key:node.is_key,
                    allowed_direction: node.allowed_direction,
                    door_direction:node.door_direction,
                    mark_i:i as usize,
                    mark_j:j as usize,
                });
            }
        }
    }
    ends
}

fn passed_trough_door(index_start : i32, index_end : i32, node : &NodeN) -> bool {
    let diff = index_start - index_end;
    match diff {
        -1 => { node.door_direction.e },
        1 => { node.door_direction.w },
        9 => { node.door_direction.n },
        -9 => { node.door_direction.s },
        _ => { false }
    }
}
fn get_num_doors_in_path(path : &Vec<i32>, nodes : &Vec<NodeN>) -> i32 {
    let mut num = 0;

    for (index, element) in path.iter().enumerate() {

       if index + 1 < path.len() &&  passed_trough_door(path[index], path[index+1],&nodes[element.clone() as usize]){
           num +=1;
       }
    }

    num
}
fn main() {

    let nodes : Arc<RwLock<Vec<Node>>> = Arc::new(RwLock::new(get_nodes()));
    let nodes_n: Arc<RwLock<Vec<NodeN>>> = Arc::new(RwLock::new(get_nodes_n(nodes.read().unwrap().to_vec())));
    let s : Arc<RwLock<State>>  = Arc::new(RwLock::new(State{
        parent: Box::new(None),
        node: nodes_n.read().unwrap()[0].clone(),
        cost: 0,
        level: 0,
        keys: 0
    }));
    let ends: Arc<RwLock<Vec<NodeN>>> = Arc::new(RwLock::new(get_end(&nodes_n.read().unwrap())));
    let mut paths_arc: Arc<RwLock<Vec<Vec<i32>>>> = Arc::new(RwLock::new(Vec::new()));

    let mut threads = Vec::new();

    for i in 0..ends.read().unwrap().len(){
        let cloned_nodes = Arc::clone(&nodes);
        let clone_nodes_n = Arc::clone(&nodes_n);
        let clone_ends = Arc::clone(&ends);
        let cloned_start = Arc::clone(&s);
        let cloned_paths_arc = Arc::clone(&paths_arc);

        threads.push(thread::spawn( move || {
                let st1 = a_star(&cloned_start.read().unwrap(), &clone_ends.read().unwrap()[i].clone(),true);
                let mut path1 = Vec::new();
                match st1 {
                    Some(s) => path1 = s.path(),
                    None => {}
                };
                let doors =  get_num_doors_in_path(&path1, &clone_nodes_n.read().unwrap());
                let mut keys = get_nodes_n_with_keys(&cloned_nodes.read().unwrap());
                let st2 = a_depth(&cloned_start.read().unwrap(), &clone_ends.read().unwrap()[i].clone(), &mut keys ,doors );
                println!("{:?}", st2.clone());
                cloned_paths_arc.write().unwrap().push(st2);
            }));
    }

    for thread in threads {
        thread.join().expect("Panic");
    }
    println!("{:?}", paths_arc.read().unwrap());

    let mut idx = 0;
    let mut i = 0;
    let mut shortest_len = paths_arc.read().unwrap().to_vec()[0].len();
    for path in paths_arc.read().unwrap().to_vec() {
        if path.len() < shortest_len{
            idx = i;
        }
        i += 1;
    }
    println!("Shortest path is : {:?}", paths_arc.read().unwrap().to_vec()[idx]);

}
