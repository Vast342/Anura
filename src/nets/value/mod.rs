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
    search::EVAL_SCALE,
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
const QA: i32 = 256;
const _QB: i32 = 64;
const QAB: i32 = QA * _QB;

#[derive(Debug)]
#[repr(C)]
#[repr(align(64))]
pub struct ValueNetwork {
    feature_weights: [i16; INPUT_SIZE * L1_SIZE * INPUT_BUCKET_COUNT],
    feature_biases: [i16; L1_SIZE],
    l2_weights: [i16; L2_SIZE * L1_SIZE * OUTPUT_BUCKET_COUNT],
    l2_biases: [i16; L2_SIZE * OUTPUT_BUCKET_COUNT],
    output_weights: [i16; L1_SIZE * OUTPUT_BUCKET_COUNT],
    output_biases: [i16; OUTPUT_BUCKET_COUNT],
}

pub const VALUE_NET: ValueNetwork = convert(unsafe { std::mem::transmute(*include_bytes!("avn_006.vn")) });

const OUTPUT_BUCKET_DIVISOR: usize = (32 + OUTPUT_BUCKET_COUNT - 1) / OUTPUT_BUCKET_COUNT;

const fn get_output_bucket(piece_count: usize) -> usize {
    (piece_count - 2) / OUTPUT_BUCKET_DIVISOR
}

#[derive(Debug, Clone)]
pub struct ValueNetworkState {
    l1_state: [i16; L1_SIZE],
    l2_state: [i16; L2_SIZE],
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

pub fn activation(x: i16) -> i32 {
    ((x as i32).max(0).min(QA)).pow(2)
}

impl ValueNetworkState {
    pub const fn new() -> Self {
        Self {
            l1_state: VALUE_NET.feature_biases,
            l2_state: [0; L2_SIZE],
        }
    }
    pub fn reset(&mut self) {
        self.state = VALUE_NET.feature_biases
        self.l2_state = VALUE_NET.
    }
    pub fn evaluate(&mut self, position: &Position, ctm: u8) -> i32 {
        self.load_position(position, ctm);
        self.forward(position.occupied().popcount() as usize)
    }
    pub fn load_position(&mut self, position: &Position, ctm: u8) {
        self.reset();
        let king = position.king_sqs[ctm as usize];
        let mut occ = position.occupied();
        while occ != Bitboard::EMPTY {
            let idx = Square(occ.pop_lsb());
            let piece = position.piece_on_square(idx);
            self.activate_feature(piece, idx, ctm, king);
        }
    }
    pub fn activate_feature(&mut self, piece: Piece, sq: Square, ctm: u8, king: Square) {
        let idx = get_feature_index(piece, sq, ctm, king);
        for hl_node in 0..L1_SIZE {
            self.state[hl_node] += VALUE_NET.feature_weights[idx * L1_SIZE + hl_node];
        }
    }
    pub fn forward(&self, piece_count: usize) -> i32 {
        let mut sum = 0;
        let output_bucket = get_output_bucket(piece_count);
        let bucket_increment = L1_SIZE * output_bucket;

        for hl_node in 0..L1_SIZE {
            sum += activation(self.state[hl_node])
                * VALUE_NET.output_weights[hl_node + bucket_increment] as i32;
        }

        (sum / QA + VALUE_NET.output_biases[output_bucket] as i32) * EVAL_SCALE as i32 / QAB
    }
}

impl Default for ValueNetworkState {
    fn default() -> Self {
        Self::new()
    }
}
