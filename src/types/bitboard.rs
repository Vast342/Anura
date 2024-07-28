#[derive(Debug, Copy, Clone, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Self = Self(0);

    pub const fn lsb(self) -> u8 {
        debug_assert!(self.0 != 0, "tried to lsb an empty bitboard");
        self.0.trailing_zeros() as u8
    }
}