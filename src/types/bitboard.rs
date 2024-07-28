use std::ops::BitXorAssign;

#[derive(Debug, Copy, Clone, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Self = Self(0);

    pub fn from_u8(sq: u8) -> Self {
        Self{0: 1 << sq as u64}
    }

    pub const fn lsb(self) -> u8 {
        debug_assert!(self.0 != 0, "tried to lsb an empty bitboard");
        self.0.trailing_zeros() as u8
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}