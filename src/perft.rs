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

use crate::{board::Board, types::MoveList};

pub fn perft(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 { return 1 };
    let mut list: MoveList = MoveList::new();
    board.get_moves(&mut list);
    let mut result: u64 = 0;
    for mov in list {
        if board.make_move(mov) {
            result += perft(board, depth - 1);
            board.undo_move();
        }
    }
    result
}