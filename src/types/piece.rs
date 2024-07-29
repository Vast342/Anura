use std::fmt;

#[derive(Debug, Copy, Clone, Default)]
pub struct Piece(pub u8);

#[derive(PartialEq, PartialOrd, Eq, Ord)]
#[repr(u8)]
pub enum Types {
    Pawn, Knight, Bishop, Rook, Queen, King, None
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
#[repr(u8)]
pub enum Colors {
    Black, White, None
}

impl Piece {
    pub const fn color(self) -> u8 {
        self.0 >> 3
    }
    pub const fn piece(self) -> u8 {
        self.0 & 0b0111
    }
    pub fn new_unchecked(piece: u8, color: u8) -> Self {
        debug_assert!(piece < 6, "invalid piece");
        debug_assert!(color < 2, "invalid color");
        Self((color << 3) | piece)
    }
}
impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut c: char = match self.piece() {
            0 => 'p',
            1 => 'n',
            2 => 'b',
            3 => 'r',
            4 => 'q',
            5 => 'k',
            _ => ' ',
        };
        if self.color() == 1 {
            c = c.to_ascii_uppercase();
        }
        write!(f, "{}", c)
    }
}