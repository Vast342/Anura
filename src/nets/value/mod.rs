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

use crate::{
    board::Position,
    mcts::search::EVAL_SCALE,
    types::{bitboard::Bitboard, piece::Piece, square::Square},
};
// value net:
// avn_008.vn
// 768->768->1x16
// l1 SCReLU
const INPUT_SIZE: usize = 768;
const INPUT_BUCKET_COUNT: usize = 1;
const HL_SIZE: usize = 768;
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
    feature_weights: [i16; INPUT_SIZE * HL_SIZE * INPUT_BUCKET_COUNT],
    feature_biases: [i16; HL_SIZE],
    output_weights: [i16; HL_SIZE * OUTPUT_BUCKET_COUNT],
    output_bias: [i16; OUTPUT_BUCKET_COUNT],
}

pub const fn transpose_output_weights(net: ValueNetwork) -> ValueNetwork {
    let mut output_weights = [0; HL_SIZE * OUTPUT_BUCKET_COUNT];
    let mut weight = 0;
    while weight < HL_SIZE {
        let mut bucket = 0;
        while bucket < OUTPUT_BUCKET_COUNT {
            let src = weight * OUTPUT_BUCKET_COUNT + bucket;
            let dst = bucket * HL_SIZE + weight;
            output_weights[dst] = net.output_weights[src];
            bucket += 1;
        }
        weight += 1;
    }
    ValueNetwork {
        feature_weights: net.feature_weights,
        feature_biases: net.feature_biases,
        output_weights,
        output_bias: net.output_bias,
    }
}

pub static VALUE_NET: ValueNetwork = transpose_output_weights(unsafe {
    std::mem::transmute::<[u8; 1205824], ValueNetwork>(*include_bytes!("net.vn"))
});

const OUTPUT_BUCKET_DIVISOR: usize = 32_usize.div_ceil(OUTPUT_BUCKET_COUNT);

const fn get_output_bucket(piece_count: usize) -> usize {
    (piece_count - 2) / OUTPUT_BUCKET_DIVISOR
}

#[derive(Debug, Clone)]
pub struct ValueNetworkState {
    state: [i16; HL_SIZE],
}

pub fn get_feature_index(piece: Piece, mut sq: Square, ctm: u8, mut king: Square) -> usize {
    let c = (piece.color() != ctm) as usize;
    if ctm == 0 {
        sq.flip();
        king.flip();
    }
    let p = piece.piece() as usize;
    INPUT_BUCKET_SCHEME[king.0 as usize] * INPUT_SIZE
        + c * COLOR_STRIDE
        + p * PIECE_STRIDE
        + sq.0 as usize
}

pub fn activation(x: i16) -> i32 {
    (x as i32).clamp(0, QA).pow(2)
}

impl ValueNetworkState {
    pub fn new() -> Self {
        Self {
            state: VALUE_NET.feature_biases,
        }
    }
    
    pub fn reset(&mut self) {
        self.state = VALUE_NET.feature_biases
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
        for hl_node in 0..HL_SIZE {
            self.state[hl_node] += VALUE_NET.feature_weights[idx * HL_SIZE + hl_node];
        }
    }
    
    pub fn forward(&self, piece_count: usize) -> i32 {
        let mut sum = 0;
        let output_bucket = get_output_bucket(piece_count);
        let bucket_increment = HL_SIZE * output_bucket;

        for hl_node in 0..HL_SIZE {
            sum += activation(self.state[hl_node])
                * VALUE_NET.output_weights[hl_node + bucket_increment] as i32;
        }

        (sum / QA + VALUE_NET.output_bias[output_bucket] as i32) * EVAL_SCALE as i32 / QAB
    }
}

impl Default for ValueNetworkState {
    fn default() -> Self {
        Self::new()
    }
}
