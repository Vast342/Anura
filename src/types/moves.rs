use std::fmt;

#[derive(Debug, Copy, Clone, Default)]
pub struct Move(pub u16);

impl Move {
    pub fn new_unchecked(start: u8, end: u8, flag: u8) -> Self {
        debug_assert!(start < 63, "invalid start square");
        debug_assert!(end < 63, "invalid end square");
        Self(((flag as u16) << 12) | ((end as u16) << 6) | (start as u16))
    }
    pub fn start(&self) -> u8 {
        (self.0 & 0b111111) as u8
    }
    pub fn end(&self) -> u8 {
        ((self.0 >> 6) & 0b111111) as u8
    }
    pub fn flag(&self) -> u8 {
        (self.0 >> 12) as u8
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = "move display will happen soon, i promise";
        write!(f, "{}", c)
    }
}