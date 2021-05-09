use std::env;
use std::{collections::HashMap, path::PathBuf};

mod engine;
mod user;
use engine::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return;
    }

    let mut engine = Engine {
        users: HashMap::new(),
    };
    let path_buff = PathBuf::from(&args[1]);

    if let Err(e) = engine.process_data(&path_buff) {
        eprintln!("{}", e);
        return;
    }

    engine.print_users();
}
