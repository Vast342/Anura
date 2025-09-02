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
// apn_012_ti.pn
// 768x4->256->1x1880
// notes:
// l1 SCReLU
// quantised
// threat inputs!

use crate::nets::policy_outs::move_index;
use crate::{
    board::Position,
    types::{moves::Move, square::Square},
};
use crate::types::bitboard::Bitboard;

const INPUT_SIZE: usize = 768 * 4;
const HL_SIZE: usize = 512;
const OUTPUT_SIZE: usize = 1880;

const QA: i16 = 128;
const QB: f32 = 128.0;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PolicyNetwork {
    pub l1_weights: [[i8; HL_SIZE]; INPUT_SIZE], // [input][hl]
    pub l1_biases: [i8; HL_SIZE],                // [hl]
    pub l2_weights: [[i8; HL_SIZE]; OUTPUT_SIZE], // [output][hl]
    pub l2_biases: [i8; OUTPUT_SIZE],            // [output]
}

pub static POLICY_NET: PolicyNetwork = unsafe { std::mem::transmute(*include_bytes!("apn.pn")) };

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
        for (i, hl) in l1.iter_mut().enumerate().take(HL_SIZE) {
            *hl = POLICY_NET.l1_biases[i] as i16;
        }
        Self { l1 }
    }

    pub fn load_position(&mut self, pos: &Position, ctm: u8) {
        self.clear();
        let king = pos.king_sqs[ctm as usize];
        let hm = if king.0 % 8 > 3 { 7 } else { 0 };
        // threats & defenses!
        let threats = pos.threats_by(ctm ^ 1);
        let defences = pos.threats_by(ctm);
        // pos -> hl
        let mut occ = pos.occupied();
        while !occ.is_empty() {
            let piece_index = occ.pop_lsb();
            let flipper = if ctm == 0 { 56 } else { 0 };
            let this_piece = pos.piece_on_square(Square(piece_index));
            let mut input = (this_piece.color() != ctm) as usize * COLOR_STRIDE
                + this_piece.piece() as usize * PIECE_STRIDE
                + (piece_index as usize ^ flipper ^ hm);
            let bit = Bitboard::from_square(Square(piece_index));
            if threats & bit != Bitboard::EMPTY {
                input += 768;
            }

            if defences & bit != Bitboard::EMPTY {
                input += 768 * 2;
            }
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
        (output as f32 / (QA as f32 * QB) + POLICY_NET.l2_biases[move_index] as f32) / QB
    }
}

// SCReLU
pub fn activation(x: i16) -> i16 {
    x.clamp(0, QA).pow(2)
}
