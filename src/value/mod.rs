use crate::{board::Position, search::EVAL_SCALE, types::{bitboard::Bitboard, piece::Piece, square::Square}};

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
// material counting eval: being used rn but not for long
//pub const PIECE_WEIGHTS: [i16; 6] = [100, 310, 330, 500, 900, 0];
// value net:
// avn_002.vn right now (VERY BAD NET)
// 768->32->1 activated by SCReLU
const INPUT_SIZE: usize = 768;
const HL_SIZE: usize = 32;

const COLOR_STRIDE: usize = 64 * 6;
const PIECE_STRIDE: usize = 64;
const QA: i32 = 256;
const _QB: i32 = 64;
const QAB: i32 = QA * _QB;

#[derive(Debug)]
#[repr(C)]
#[repr(align(64))]
pub struct ValueNetwork {
    feature_weights: [i16; INPUT_SIZE * HL_SIZE],
    feature_biases:  [i16; HL_SIZE],
    output_weights:  [i16; HL_SIZE],
    output_bias: i16,
}

pub const NET: ValueNetwork = unsafe { std::mem::transmute(*include_bytes!("avn_002.vn")) };

#[derive(Debug, Clone)]
pub struct ValueNetworkState {
    state: [i16; HL_SIZE],
}

pub const fn get_feature_index(piece: Piece, sq: Square) -> usize{
    let c = 1 - piece.color() as usize;
    let p = piece.piece() as usize;
    return c * COLOR_STRIDE + p * PIECE_STRIDE + sq.0 as usize;
}

pub fn activation(x: i16) -> i32 {
    ((x as i32).max(0).min(QA)).pow(2)
}

impl ValueNetworkState {
    pub const fn new() -> Self {
        Self{state: NET.feature_biases}
    }
    pub fn reset(&mut self) {
        self.state = NET.feature_biases
    }
    pub fn evaluate(&mut self, position: &Position) -> i32 {
        self.load_position(position);
        self.forward()
    }
    pub fn load_position(&mut self, position: &Position) {
        self.reset();
        let mut occ = position.occupied();
        while occ != Bitboard::EMPTY {
            let idx = Square(occ.pop_lsb());
            let piece = position.piece_on_square(idx);
            self.activate_feature(piece, idx);
        }
    }
    pub fn activate_feature(&mut self, piece: Piece, sq: Square) {
        let idx = get_feature_index(piece, sq);
        for hl_node in 0..HL_SIZE {
            self.state[hl_node] += NET.feature_weights[idx * HL_SIZE + hl_node];
        }
    }
    pub fn forward(&self) -> i32 {
        let mut sum = 0;

        for hl_node in 0..HL_SIZE {
            sum += activation(self.state[hl_node]) * NET.output_weights[hl_node] as i32;
        }

        (sum / QA + NET.output_bias as i32) * EVAL_SCALE as i32 / QAB
    }
}

impl Default for ValueNetworkState {
    fn default() -> Self {
        Self::new()
    }
}