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
pub mod loader;
use loader::convert;
use crate::{
    board::Position,
    types::{bitboard::Bitboard, piece::Piece, square::Square},
};

// value net:
// avn_007.vn
// 768->512->(16->1)x16 activated by SCReLU
const INPUT_SIZE: usize = 768;
const INPUT_BUCKET_COUNT: usize = 1;
const L1_SIZE: usize = 512;
const L2_SIZE: usize = 16;
const OUTPUT_BUCKET_COUNT: usize = 16;

#[rustfmt::skip]
const INPUT_BUCKET_SCHEME: [usize; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
];

const COLOR_STRIDE: usize = 64 * 6;
const PIECE_STRIDE: usize = 64;

#[derive(Debug)]
#[repr(C)]
#[repr(align(64))]
pub struct ValueNetwork {
    feature_weights: [f32; INPUT_SIZE * L1_SIZE * INPUT_BUCKET_COUNT],
    feature_biases: [f32; L1_SIZE],
    l2_weights: [[[f32; L2_SIZE]; OUTPUT_BUCKET_COUNT]; L1_SIZE],
    l2_biases: [[f32; L2_SIZE]; OUTPUT_BUCKET_COUNT],
    output_weights: [f32; L2_SIZE * OUTPUT_BUCKET_COUNT],
    output_biases: [f32; OUTPUT_BUCKET_COUNT],
}

pub const VALUE_NET: ValueNetwork = convert(unsafe { std::mem::transmute(*include_bytes!("avn_007.vn")) });

const OUTPUT_BUCKET_DIVISOR: usize = (32 + OUTPUT_BUCKET_COUNT - 1) / OUTPUT_BUCKET_COUNT;

const fn get_output_bucket(piece_count: usize) -> usize {
    (piece_count - 2) / OUTPUT_BUCKET_DIVISOR
}

#[derive(Debug, Clone)]
pub struct ValueNetworkState {
    l1_state: [f32; L1_SIZE],
    l2_state: [f32; L2_SIZE],
}

pub fn get_feature_index(piece: Piece, mut sq: Square, ctm: u8, mut king: Square) -> usize {
    let c = (piece.color() != ctm) as usize;
    if ctm == 0 {
        sq.flip();
        king.flip();
    }
    let p = piece.piece() as usize;
    return INPUT_BUCKET_SCHEME[king.0 as usize] * INPUT_SIZE
        + c * COLOR_STRIDE
        + p * PIECE_STRIDE
        + sq.0 as usize;
}

pub fn activation(x: f32) -> f32 {
    ((x).max(0.0).min(1.0)).powf(2.0)
}

impl ValueNetworkState {
    pub const fn new() -> Self {
        Self {
            l1_state: VALUE_NET.feature_biases,
            l2_state: [0.0; L2_SIZE],
        }
    }
    pub fn reset(&mut self, output_bucket: usize) {
        self.l1_state = VALUE_NET.feature_biases;
        self.l2_state = VALUE_NET.l2_biases[output_bucket];
    }
    pub fn evaluate(&mut self, position: &Position, ctm: u8) -> f32 {
        self.load_position(position, ctm);
        self.l1_to_l2(position.occupied().popcount() as usize);
        self.forward(position.occupied().popcount() as usize)
    }
    pub fn load_position(&mut self, position: &Position, ctm: u8) {
        let mut occ = position.occupied();
        let output_bucket = get_output_bucket(occ.popcount() as usize);
        self.reset(output_bucket);
        let king = position.king_sqs[ctm as usize];
        while occ != Bitboard::EMPTY {
            let idx = Square(occ.pop_lsb());
            let piece = position.piece_on_square(idx);
            self.activate_feature(piece, idx, ctm, king);
        }
    }
    pub fn activate_feature(&mut self, piece: Piece, sq: Square, ctm: u8, king: Square) {
        let idx = get_feature_index(piece, sq, ctm, king);
        for hl_node in 0..L1_SIZE {
            self.l1_state[hl_node] += VALUE_NET.feature_weights[idx * L1_SIZE + hl_node];
        }
    }
    pub fn l1_to_l2(&mut self, piece_count: usize) {
        let output_bucket = get_output_bucket(piece_count);
        for l1_node in 0..L1_SIZE {
            for l2_node in 0..L2_SIZE {
                self.l2_state[l2_node] += activation(self.l1_state[l1_node]) * VALUE_NET.l2_weights[l1_node][output_bucket][l2_node];
            }
        }
    }
    pub fn forward(&self, piece_count: usize) -> f32 {
        let mut sum = 0.0;
        let output_bucket = get_output_bucket(piece_count);
        let bucket_increment = L2_SIZE * output_bucket;

        for hl_node in 0..L2_SIZE {
            sum += activation(self.l2_state[hl_node]) * VALUE_NET.output_weights[hl_node + bucket_increment];
        }

        return 1.0 / (1.0 + (-(sum + VALUE_NET.output_biases[output_bucket])).exp());
    }
}

impl Default for ValueNetworkState {
    fn default() -> Self {
        Self::new()
    }
}
