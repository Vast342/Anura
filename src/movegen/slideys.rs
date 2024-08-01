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

use super::lookups::SLIDEY_PIECE_RAYS;

// slidey pieces

// Rooks
// Classical Approach
#[must_use] pub fn get_rook_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    let mut total_attacks: Bitboard = Bitboard(0);
    for (dir, item) in SLIDEY_PIECE_RAYS.iter().enumerate().take(4) {
        let mut current_attack: Bitboard = Bitboard(item[sq.as_usize()]);
        
        if (current_attack & occupied) != Bitboard(0) {
            if (dir & 1) == 0 {
                current_attack ^= Bitboard(item[(current_attack & occupied).lsb() as usize]);
            } else {
                current_attack ^= Bitboard(item[63 - (current_attack & occupied).msb() as usize]);
            }
        }
        total_attacks |= current_attack;
    }
    total_attacks
}
// Magics

// Pext?

// Bishops
// Classical Approach
#[must_use] pub fn get_bishop_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    let mut total_attacks: Bitboard = Bitboard(0);
    for (dir, item) in SLIDEY_PIECE_RAYS.iter().enumerate().skip(4) {
        let mut current_attack: Bitboard = Bitboard(item[sq.as_usize()]);
    
        if (current_attack & occupied) != Bitboard(0) {
            if (dir & 1) == 0 {
                current_attack ^= Bitboard(item[(current_attack & occupied).lsb() as usize]);
            } else {
                current_attack ^= Bitboard(item[63 - (current_attack & occupied).msb() as usize]);
            }
        }
        total_attacks |= current_attack;
    }
    total_attacks
}
// Magics

// Pext?