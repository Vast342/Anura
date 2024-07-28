#[derive(Debug, Copy, Clone, Default)]
pub struct Piece(pub u8);

#[derive(PartialEq, PartialOrd, Eq, Ord)]
#[repr(u8)]
pub enum PieceType {
    Pawn, Knight, Bishop, Rook, Queen, King, None
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
#[repr(u8)]
pub enum PieceColor {
    Black, White, None
}

pub fn convert_pt(x: u8) -> PieceType {
    match x {
        0 => PieceType::Pawn,
        1 => PieceType::Knight,
        2 => PieceType::Bishop,
        3 => PieceType::Rook,
        4 => PieceType::Queen,
        5 => PieceType::King,
        _ => PieceType::None,
    }
}

pub fn convert_pc(x: u8) -> PieceColor {
    match x {
        0 => PieceColor::Black,
        1 => PieceColor::White,
        _ => PieceColor::None,
    }
}

impl Piece {
    pub fn color(self) -> PieceColor {
        convert_pc(self.0 >> 3)
    }
    pub fn piece(self) -> PieceType {
        convert_pt(self.0 & 0b0111)
    }
    pub fn new_unchecked(color: PieceColor, piece: PieceType) -> Self {
        debug_assert!(piece == PieceType::None, "invalid piece");
        Self(((color as u8) << 3) & piece as u8)
    }
}