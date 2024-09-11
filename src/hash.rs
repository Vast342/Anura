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

use crate::{
    prng::fill_array,
    types::{piece::Piece, square::Square},
};

const PIECE_SQUARE_SIZE: usize = 14 * 64;
const CTM_SIZE: usize = 1;
const PIECE_SQUARE_STRIDE: usize = 0;
const CTM_STRIDE: usize = PIECE_SQUARE_STRIDE + PIECE_SQUARE_SIZE;

const TOTAL_SIZE: usize = PIECE_SQUARE_SIZE + CTM_SIZE;

//                                                read this as "tastelesscascade"
const ZOBRIST_VALUES: [u64; TOTAL_SIZE] = fill_array();

pub fn zobrist_psq(piece: Piece, sq: Square) -> u64 {
    ZOBRIST_VALUES[PIECE_SQUARE_STRIDE + sq.0 as usize * 14 + piece.0 as usize]
}

pub fn zobrist_ctm() -> u64 {
    ZOBRIST_VALUES[CTM_STRIDE]
}
