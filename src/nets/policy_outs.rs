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

use crate::movegen::lookups::{DESTINATIONS, OFFSET_TABLE};

pub fn move_index(piece: u8, from: u8, to: u8) -> usize {
    let to_bb = 1u64 << to as u64;
    let mask = DESTINATIONS[from as usize][piece as usize];

    let res = (mask & (to_bb - 1)).count_ones() as usize;

    OFFSET_TABLE[from as usize][piece as usize] + res
}
