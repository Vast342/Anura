#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

pub mod uci;
pub mod board;
pub mod types;
pub mod eval;
pub mod movegen;

//use crate::board::BoardState;
use crate::uci::UciManager;
use std::env;

fn main() {
    // initialize();
    env::set_var("RUST_BACKTRACE", "1");
    let mut manager: UciManager = UciManager::new();
    loop {
        if !manager.get_command() {
            break;
        }
    }
}

// pext is _pext_u64