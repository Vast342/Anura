use std::ops::{BitXorAssign, BitAnd, BitOrAssign, BitOr, Shl, Shr, Not};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Bitboard(pub u64);

use super::square::Square;

// a mask for a single file on the board
pub const FILEMASK: u64 = 0b100000001000000010000000100000001000000010000000100000001;
// a mask for a single rank on the board
pub const RANKMASK: u64 = 0b11111111;

impl Bitboard {

    pub const EMPTY: Self = Self(0);

    pub fn from_square(sq: Square) -> Self {
        Self{0: 1 << sq.0 as u64}
    }
    pub fn from_rank(rank: u8) -> Self {
        Self(RANKMASK << (8 * rank))
    }
    pub fn from_file(file: u8) -> Self {
        Self(FILEMASK << file)
    }

    pub const fn lsb(&self) -> u8 {
        debug_assert!(self.0 != 0, "tried to lsb an empty bitboard");
        self.0.trailing_zeros() as u8
    }
    pub const fn msb(&self) -> u8 {
        debug_assert!(self.0 != 0, "tried to lsb an empty bitboard");
        self.0.leading_zeros() as u8
    }
    pub fn pop_lsb(&mut self) -> u8 {
        let lsb: u8 = self.lsb();
        self.0 &= self.0 - 1;
        lsb
    }
    pub const fn popcount(&self) -> u32 {
        self.0.count_ones()
    }
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }
    pub const fn has_bits(&self) -> bool {
        self.0 != 0
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0 )
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0 )
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
