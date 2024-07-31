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
use crate::types::bitboard::Bitboard;
use crate::types::square::Square;
use super::lookups::PAWN_ATTACKS;

// input: pawn bitboard, unoccupied bitboard, and ctm
// output: tuple of single and double pawn pushes
#[must_use] pub fn get_pawn_pushes_setwise(pawns: Bitboard, empties: Bitboard, ctm: u8) -> (Bitboard, Bitboard) {
    let single: Bitboard = if ctm == 0 { (pawns >> 8) & empties } else { (pawns << 8) & empties };
    let mut double = single & Bitboard::from_rank(if ctm == 1 { 2 } else { 5 });
    if ctm == 0 {
        double = (double >> 8) & empties;
    } else {
        double = (double << 8) & empties;
    }
    (single, double)
}

// input: pawn bitboard, opponent bitboard, and ctm
// output: tuple of left and right pawn pushes
#[must_use] pub fn get_pawn_attacks_setwise(pawns: Bitboard, capturable: Bitboard, ctm: u8) -> (Bitboard, Bitboard) {
    let mut left_attacks = if ctm == 0 { pawns >> 9 } else { pawns << 7 };
    let mut right_attacks= if ctm == 0 { pawns >> 7 } else { pawns << 9 };
    left_attacks  &= !Bitboard::from_file(7);
    right_attacks &= !Bitboard::from_file(0);
    left_attacks  &= capturable;
    right_attacks &= capturable;
    (left_attacks, right_attacks)
}

// single square pawn capture lookups
#[must_use] pub const fn get_pawn_attacks_lookup(sq: Square, ctm: u8) -> Bitboard {
    Bitboard(PAWN_ATTACKS[ctm as usize][sq.as_usize()])
}