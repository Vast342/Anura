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
use std::arch::x86_64::_pext_u64;

use crate::types::bitboard::Bitboard;
use crate::types::square::Square;

use super::lookups::{BISHOP_MASKS, ROOK_MASKS};

// slidey pieces

// todo: magic bitboards backup
// would prefer to generate my own magics instead of using c#larity's again

pub const MAX_ROOK_ENTRIES: usize = 4096;
pub const MAX_BISHOP_ENTRIES: usize = 512;
pub const ROOK_TABLE_SIZE: usize = 2097152;
pub const BISHOP_TABLE_SIZE: usize = 262144;

static ROOK_MOVES: [[u64; MAX_ROOK_ENTRIES]; 64] =
    unsafe { std::mem::transmute(*include_bytes!("tables/rooks.bin")) };

static BISHOP_MOVES: [[u64; MAX_BISHOP_ENTRIES]; 64] =
    unsafe { std::mem::transmute(*include_bytes!("tables/bishops.bin")) };

#[must_use]
#[inline(always)]
pub fn get_rook_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    Bitboard(ROOK_MOVES[sq.as_usize()][get_rook_index_pext(sq, occupied)])
}

#[inline(always)]
fn get_rook_index_pext(sq: Square, occupied: Bitboard) -> usize {
    unsafe { _pext_u64(occupied.as_u64(), ROOK_MASKS[sq.as_usize()]) as usize }
}

#[must_use]
#[inline(always)]
pub fn get_bishop_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    Bitboard(BISHOP_MOVES[sq.as_usize()][get_bishop_index_pext(sq, occupied)])
}

#[inline(always)]
fn get_bishop_index_pext(sq: Square, occupied: Bitboard) -> usize {
    unsafe { _pext_u64(occupied.as_u64(), BISHOP_MASKS[sq.as_usize()]) as usize }
}
