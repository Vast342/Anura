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
use super::lookups::{KING_ATTACKS, KNIGHT_ATTACKS};
use crate::types::bitboard::Bitboard;
use crate::types::square::Square;

// Knights
#[must_use]
pub const fn get_knight_attacks(sq: Square) -> Bitboard {
    Bitboard(KNIGHT_ATTACKS[sq.as_usize()])
}

// Kings
#[must_use]
pub const fn get_king_attacks(sq: Square) -> Bitboard {
    Bitboard(KING_ATTACKS[sq.as_usize()])
}
