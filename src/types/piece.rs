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
    pub fn new_unchecked(color: u8, piece: u8) -> Self {
        debug_assert!(piece < 6, "invalid piece");
        debug_assert!(color < 2, "invalid color");
        Self((color << 3) & piece)
    }
}