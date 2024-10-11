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
// policy "net":
// apn_003.pn
// (768->1)x384
// extra notes:
// to be quantised soon
// first 2 failed because of trainer troubles

mod loader;
use loader::{load, QA};

use crate::{
    board::Position,
    types::{moves::Move, piece::Piece, square::Square},
};
pub const INPUT_SIZE: usize = 768;
pub const SUBNET_COUNT: usize = 384;
pub const OW_SIZE: usize = INPUT_SIZE * SUBNET_COUNT;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
#[repr(align(64))]
pub struct PolicyNetwork {
    pub output_weights: [i32; OW_SIZE],
    pub output_biases: [i32; SUBNET_COUNT],
}

impl Default for PolicyNetwork {
    fn default() -> Self {
        Self{ output_weights: [0; OW_SIZE], output_biases: [0; SUBNET_COUNT]}
    }
}

pub fn get_policy_net() -> Box<PolicyNetwork> {
    load(unsafe { std::mem::transmute(*include_bytes!("apn_003.pn")) })
}

/* will need for more layers but not rn with my glorified psqts
pub struct PolicyNetworkState{

}*/

pub const PIECE_STEP: usize = 64;
pub const COLOR_STEP: usize = 64 * 6;

pub fn calculate_index(move_piece: Piece, move_to: usize, piece: Piece, square: usize) -> usize {
    let move_number = PIECE_STEP * move_piece.piece() as usize + move_to;
    let input_number =
        COLOR_STEP * piece.color() as usize + PIECE_STEP * piece.piece() as usize + square;
    INPUT_SIZE * move_number + input_number
    // highest possible would be uhhhh
    // 768 * (64 * 5 + 63) + (384 + 64 * 5 + 63)
}

impl PolicyNetwork {
    pub fn get_score(&self, pos: &Position, mov: Move) -> f32 {
        let piece = pos.piece_on_square(Square(mov.from()));
        let to = mov.to();
        // infer
        let mut result = self.output_biases[(64 * piece.0 + to) as usize];
        for piece_index in 0..64 {
            let this_piece = pos.piece_on_square(Square(piece_index));
            if this_piece != Piece(6) {
                let index = calculate_index(piece, to as usize, this_piece, piece_index as usize);
                result += self.output_weights[index];
            }
        }
        result as f32 / QA
    }
}
