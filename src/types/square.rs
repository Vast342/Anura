use std::ops::AddAssign;

#[derive(Debug, Copy, Clone, Default)]
pub struct Square(pub u8);

impl Square {
    pub fn flip(&mut self) {
        self.0 ^= 56;
    }
    pub const fn rank(&self) -> u8 /* i refuse to write a rank wrapper */ {
        self.0 / 8
    }
    pub const fn file(&self) -> u8 /* i refuse to write a file wrapper */ {
        self.0 & 0b111
    }
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
    pub fn from_rf(rank: u8, file: u8) -> Self {
        Self(rank * 8 + file)
    }
}

impl AddAssign for Square {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}