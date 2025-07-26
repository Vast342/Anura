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
// policy net:
// apn_008.pn
// 768->256->1x1880
// notes:
// l1 SCReLU
// quantised

mod outs;
use outs::move_index;

use crate::{
    board::Position,
    types::{moves::Move, square::Square},
};
const INPUT_SIZE: usize = 768;
const HL_SIZE: usize = 256;
const OUTPUT_SIZE: usize = 1880;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PolicyNetwork {
    pub l1_weights: [[i16; HL_SIZE]; INPUT_SIZE], // [input][hl]
    pub l1_biases: [i16; HL_SIZE],                // [hl]
    pub l2_weights: [[i16; HL_SIZE]; OUTPUT_SIZE], // [output][hl]
    pub l2_biases: [i32; OUTPUT_SIZE],            // [output]
}

pub const fn transpose_output_weights(net: PolicyNetwork) -> PolicyNetwork {
    let mut transposed_l2_weights = [[0; HL_SIZE]; OUTPUT_SIZE];
    let mut output_idx = 0;
    while output_idx < OUTPUT_SIZE {
        let mut hl_idx = 0;
        while hl_idx < HL_SIZE {
            let linear_idx = hl_idx * OUTPUT_SIZE + output_idx;
            let struct_output_idx = linear_idx / HL_SIZE;
            let struct_hl_idx = linear_idx % HL_SIZE;

            if struct_output_idx < OUTPUT_SIZE && struct_hl_idx < HL_SIZE {
                transposed_l2_weights[output_idx][hl_idx] =
                    net.l2_weights[struct_output_idx][struct_hl_idx];
            }
            hl_idx += 1;
        }
        output_idx += 1;
    }

    PolicyNetwork {
        l1_weights: net.l1_weights,
        l1_biases: net.l1_biases,
        l2_weights: transposed_l2_weights,
        l2_biases: net.l2_biases,
    }
}

pub static POLICY_NET: PolicyNetwork =
    transpose_output_weights(unsafe { std::mem::transmute(*include_bytes!("net.pn")) });

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
        let mut l1 = [0; HL_SIZE];
        for i in 0..HL_SIZE {
            l1[i] = POLICY_NET.l1_biases[i] as i16;
        }
        Self { l1 }
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
                self.l1[hl_node] += POLICY_NET.l1_weights[input][hl_node] as i16;
            }
        }
    }

    pub fn clear(&mut self) {
        for i in 0..HL_SIZE {
            self.l1[i] = POLICY_NET.l1_biases[i] as i16;
        }
    }

    pub fn get_score(&self, mov: Move, ctm: u8, king: Square) -> f32 {
        let move_index = move_index(ctm, mov, king);
        let mut output: i32 = 0;
        // hl -> output
        for hl_node in 0..HL_SIZE {
            output += (POLICY_NET.l2_weights[move_index][hl_node] as i32)
                * (activation(self.l1[hl_node]) as i32);
        }
        (output as f32 / 128.0 + POLICY_NET.l2_biases[move_index] as f32)
            / (128.0 * 128.0)
    }
}

// SCReLU
pub fn activation(x: i16) -> i16 {
    x.clamp(0, 128).pow(2)
}
