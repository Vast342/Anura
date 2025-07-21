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
// policy "net":
// apn_008.pn
// 768->256->1x1880
// notes:
// l1 SCReLU
// quantised
// this will probably be the last new net that is stored in this repo

mod outs;
use outs::move_index;

use crate::{
    board::Position,
    types::{moves::Move, square::Square},
};
const INPUT_SIZE: usize = 768;
const HL_SIZE: usize = 256;
const OUTPUT_SIZE: usize = 1880;

const QA: i16 = 128;
const QB: i16 = 128;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PolicyNetwork {
    pub l1_weights: [[i16; HL_SIZE]; INPUT_SIZE], // [input][hl]
    pub l1_biases: [i16; HL_SIZE],                // [hl]
    pub l2_weights: [[i16; OUTPUT_SIZE]; HL_SIZE], // [hl][output]
    pub l2_biases: [i32; OUTPUT_SIZE],            // [output]
}

pub static POLICY_NET: PolicyNetwork =
    unsafe { std::mem::transmute::<[u8; 1363808], PolicyNetwork>(*include_bytes!("net.pn")) };

#[derive(Debug, Clone)]
pub struct PolicyAccumulator {
    pub l1: [i16; HL_SIZE],
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
        let king = pos.king_sqs[ctm as usize];
        let hm = if king.0 % 8 > 3 { 7 } else { 0 };
        // pos -> hl
        let mut occ = pos.occupied();
        while !occ.is_empty() {
            let piece_index = occ.pop_lsb();
            let flipper = if ctm == 0 { 56 } else { 0 };
            let this_piece = pos.piece_on_square(Square(piece_index));
            let input = (this_piece.color() != ctm) as usize * COLOR_STRIDE
                + this_piece.piece() as usize * PIECE_STRIDE
                + (piece_index as usize ^ flipper ^ hm);
            for hl_node in 0..HL_SIZE {
                self.l1[hl_node] += POLICY_NET.l1_weights[input][hl_node];
            }
        }
    }
    pub fn clear(&mut self) {
        self.l1 = POLICY_NET.l1_biases;
    }
    pub fn get_score(&self, mov: Move, ctm: u8, king: Square) -> f32 {
        let move_index = move_index(ctm, mov, king);
        let mut output: i32 = 0;
        // hl -> output
        for hl_node in 0..HL_SIZE {
            output += (POLICY_NET.l2_weights[hl_node][move_index] as i32)
                * (activation(self.l1[hl_node]) as i32);
        }
        (output as f32 / QA as f32 + POLICY_NET.l2_biases[move_index] as f32)
            / (QA as f32 * QB as f32)
    }
}

// SCReLU
pub fn activation(x: i16) -> i16 {
    x.clamp(0, QA).pow(2)
}
