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
// (768->1)x1880
// notes:
// back to unquantised, will need to quantise later

mod outs;
use outs::move_index;

use crate::{
    board::Position,
    types::{moves::Move, piece::Piece, square::Square},
};
const INPUT_SIZE: usize = 768;
const HL_SIZE: usize = 32;
const OUTPUT_SIZE: usize = 1880;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PolicyNetwork {
    pub l1_weights: [[f32; HL_SIZE]; INPUT_SIZE], // [input][hl]
    pub l1_biases: [f32; HL_SIZE],                // [hl]
    pub l2_weights: [[f32; OUTPUT_SIZE]; HL_SIZE], // [hl][output]
    pub l2_biases: [f32; OUTPUT_SIZE],            // [output]
}

pub static POLICY_NET: PolicyNetwork =
    unsafe { std::mem::transmute(*include_bytes!("apn_004.pn")) };

#[derive(Debug, Clone)]
pub struct PolicyAccumulator {
    pub l1: [f32; HL_SIZE],
}

impl Default for PolicyAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

const COLOR_STRIDE: usize = 64 * 6;
const PIECE_STRIDE: usize = 64;

impl PolicyAccumulator {
    fn new() -> Self {
        Self {
            l1: POLICY_NET.l1_biases,
        }
    }
    pub fn load_position(&mut self, pos: &Position, ctm: u8) {
        self.clear();
        // pos -> hl
        // could be more efficient with poplsb loop through occupied bitboard
        for piece_index in 0..64 {
            let flipper = if ctm == 0 { 56 } else { 0 };
            let this_piece = pos.piece_on_square(Square(piece_index));
            if this_piece != Piece(6) {
                let input = (this_piece.color() != ctm) as usize * COLOR_STRIDE
                    + this_piece.piece() as usize * PIECE_STRIDE
                    + piece_index as usize
                    ^ flipper;
                for hl_node in 0..HL_SIZE {
                    self.l1[hl_node] += POLICY_NET.l1_weights[input][hl_node];
                }
            }
        }
    }
    pub fn clear(&mut self) {
        self.l1 = POLICY_NET.l1_biases;
    }
    pub fn get_score(&self, mov: Move, ctm: u8) -> f32 {
        let move_index = move_index(ctm, mov);
        let mut output = POLICY_NET.l2_biases[move_index];
        // hl -> output
        for hl_node in 0..HL_SIZE {
            output += POLICY_NET.l2_weights[hl_node][move_index] * activation(self.l1[hl_node]);
        }
        output
    }
}

// CReLU
pub fn activation(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}
