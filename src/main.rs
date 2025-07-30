/*
    Anura
    Copyright (C) 2025 Joseph Pasfield

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
#![allow(
    clippy::missing_panics_doc,
    clippy::cargo_common_metadata,
    clippy::cast_possible_truncation,
    clippy::single_match,
)]

pub mod board;
#[cfg(feature = "datagen")]
pub mod datagen;
pub mod hash;
pub mod mcts;
pub mod movegen;
pub mod nets;
pub mod perft;
pub mod prng;
pub mod rays;
pub mod tunable;
pub mod types;
pub mod uci;

#[cfg(feature = "datagen")]
use datagen::datagen_main;
#[cfg(feature = "datagen")]
use datagen::gen_fens;
use movegen::lookups::initialize;

use crate::uci::Manager;
use std::env;

fn main() {
    initialize();
    let args: Vec<String> = env::args().collect();
    let mut manager: Manager = Manager::new();
    if args.len() > 1 {
        if args[1] == "bench" {
            manager.bench();
        } else if args[1] == "datagen" {
            #[cfg(feature = "datagen")]
            datagen_main(args);
        } else if args[1] == "perftsuite" && cfg!(feature = "perftsuite") {
            manager.perft_suite();
        } else if args[1].split_ascii_whitespace().collect::<Vec<&str>>()[0] == "genfens" {
            #[cfg(feature = "datagen")]
            gen_fens(args);
        }
    } else {
        loop {
            if !manager.get_command() {
                break;
            }
        }
    }
}

// pext is _pext_u64
