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

use crate::types::{bitboard::Bitboard, square::Square};

#[repr(C)]
#[derive(Clone, Copy)]
struct Mask {
    diagonal: u64,
    antidiagonal: u64,
    vertical: u64,
}

#[repr(C)]
struct LookupTables {
    masks: [Mask; 64],
    rank_attack: [u8; 512],
}

#[inline(always)]
const fn bit_bswap(b: u64) -> u64 {
    b.swap_bytes()
}

static TABLES: LookupTables = unsafe { std::mem::transmute(*include_bytes!("hq_tables.bin")) };
static MASKS: [Mask; 64] = TABLES.masks;
static RANK_ATTACK: [u8; 512] = TABLES.rank_attack;

#[inline(always)]
fn attack(pieces: u64, x: u32, mask: u64) -> u64 {
    let o = pieces & mask;
    ((o.wrapping_sub(1u64 << x)) ^ bit_bswap(bit_bswap(o).wrapping_sub(0x8000_0000_0000_0000u64 >> x))) & mask
}

#[inline(always)]
fn horizontal_attack(pieces: u64, x: u32) -> u64 {
    let file_mask = x & 7;
    let rank_mask = x & 56;
    let o = (pieces >> rank_mask) & 126;
    (RANK_ATTACK[(o * 4 + file_mask as u64) as usize] as u64) << rank_mask
}

#[inline(always)]
fn vertical_attack(occ: u64, sq: u32) -> u64 {
    attack(occ, sq, MASKS[sq as usize].vertical)
}

#[inline(always)]
fn diagonal_attack(occ: u64, sq: u32) -> u64 {
    attack(occ, sq, MASKS[sq as usize].diagonal)
}

#[inline(always)]
fn antidiagonal_attack(occ: u64, sq: u32) -> u64 {
    attack(occ, sq, MASKS[sq as usize].antidiagonal)
}

#[inline(always)]
pub fn get_bishop_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    Bitboard(
        diagonal_attack(occ.0, sq.0 as u32) |
        antidiagonal_attack(occ.0, sq.0 as u32)
    )
}

#[inline(always)]
pub fn get_rook_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    Bitboard(
        vertical_attack(occ.0, sq.0 as u32) |
        horizontal_attack(occ.0, sq.0 as u32)
    )
}