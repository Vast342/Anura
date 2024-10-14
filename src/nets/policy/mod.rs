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
// apn_001.pn
// (768->1)x384

use crate::{
    board::Position,
    types::{moves::Move, piece::Piece, square::Square},
};
const INPUT_SIZE: usize = 768;
const OUTPUT_SIZE: usize = 384;
const OW_SIZE: usize = INPUT_SIZE * OUTPUT_SIZE;

const QA: f32 = 512.0;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
#[repr(align(64))]
pub struct PolicyNetwork {
    pub output_weights: [i16; OW_SIZE],
    pub output_biases: [i16; OUTPUT_SIZE],
}

pub static POLICY_NET: PolicyNetwork = unsafe { std::mem::transmute(*include_bytes!("apn_003.pn")) };

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

pub fn get_score(pos: &Position, mov: Move) -> f32 {
    let piece = pos.piece_on_square(Square(mov.from()));
    let to = mov.to();
    // infer
    let mut result = POLICY_NET.output_biases[(64 * piece.0 + to) as usize];
    for piece_index in 0..64 {
        let this_piece = pos.piece_on_square(Square(piece_index));
        if this_piece != Piece(6) {
            let index = calculate_index(piece, to as usize, this_piece, piece_index as usize);
            result += POLICY_NET.output_weights[index];
        }
    }
    result as f32 / QA
}
