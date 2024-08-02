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
use rand::Rng;

use crate::{board::Board, types::{moves::Move, MoveList}};

pub fn random(mut board: Board) -> Move {
    let mut list: MoveList = MoveList::new();
    board.get_moves(&mut list);
    loop {
        let index = rand::thread_rng().gen_range(0..list.len());
        if board.make_move(list[index]) {
            board.undo_move();
            return list[index]
        }
    }
}