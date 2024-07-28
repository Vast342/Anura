#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

pub mod uci;
pub mod board;
pub mod types;

//use crate::board::BoardState;
use crate::uci::uci_main;

fn main() {
    loop {
        uci_main();
    }
}

// pext is _pext_u64