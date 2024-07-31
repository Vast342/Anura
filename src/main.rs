/*
    Anura
    Copyright (C) 2024 Joseph Pasfield

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

#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::missing_panics_doc, clippy::cargo_common_metadata)]

pub mod uci;
pub mod board;
pub mod types;
pub mod eval;
pub mod movegen;

use crate::uci::Manager;
use std::env;

fn main() {
    // initialize();
    env::set_var("RUST_BACKTRACE", "1");
    let mut manager: Manager = Manager::new();
    loop {
        if !manager.get_command() {
            break;
        }
    }
}

// pext is _pext_u64