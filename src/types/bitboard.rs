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

use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Bitboard(pub u64);

use super::square::Square;

// a mask for a single file on the board
pub const FILEMASK: u64 = 0b1_0000_0001_0000_0001_0000_0001_0000_0001_0000_0001_0000_0001_0000_0001;
// a mask for a single rank on the board
pub const RANKMASK: u64 = 0b1111_1111;

impl Bitboard {
    pub const EMPTY: Self = Self(0);

    #[must_use]
    pub const fn from_square(sq: Square) -> Self {
        Self(1 << sq.0)
    }

    #[must_use]
    pub const fn from_rank(rank: u8) -> Self {
        Self(RANKMASK << (8 * rank))
    }

    #[must_use]
    pub const fn from_file(file: u8) -> Self {
        Self(FILEMASK << file)
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn lsb(&self) -> u8 {
        debug_assert!(self.0 != 0, "tried to lsb an empty bitboard");
        self.0.trailing_zeros() as u8
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn msb(&self) -> u8 {
        debug_assert!(self.0 != 0, "tried to msb an empty bitboard");
        self.0.leading_zeros() as u8
    }

    pub fn pop_lsb(&mut self) -> u8 {
        let lsb: u8 = self.lsb();
        self.0 &= self.0 - 1;
        lsb
    }

    #[must_use]
    pub const fn popcount(&self) -> u32 {
        self.0.count_ones()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn is_not_empty(&self) -> bool {
        self.0 != 0
    }

    #[must_use]
    pub const fn has_bits(&self) -> bool {
        self.0 != 0
    }

    #[must_use]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn contains_multiple(self) -> bool {
        (self.0 & self.0.wrapping_sub(1)) != 0
    }

    #[must_use]
    pub const fn contains_one(self) -> bool {
        !self.is_empty() && !self.contains_multiple()
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Shr<u8> for Bitboard {
    type Output = Self;

    fn shr(self, rhs: u8) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl Shl<u8> for Bitboard {
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
