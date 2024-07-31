use std::fmt;

use crate::board::SQUARE_NAMES;

#[derive(Debug, Copy, Clone, Default)]
pub struct Move(pub u16);

#[derive(PartialEq, PartialOrd, Eq, Ord)]
#[repr(u8)]
pub enum Flag {
    Normal,
    WKCastle,
    WQCastle,
    BKCastle,
    BQCastle,
    EnPassant,
    DoublePush,
    KnightPromo,
    BishopPromo,
    RookPromo,
    QueenPromo,
}

impl Flag {
    fn from_u8(value: u8) -> Flag {
        match value {
            0 => Flag::Normal,
            1 => Flag::WKCastle,
            2 => Flag::WQCastle,
            3 => Flag::BKCastle,
            4 => Flag::BQCastle,
            5 => Flag::EnPassant,
            6 => Flag::DoublePush,
            7 => Flag::KnightPromo,
            8 => Flag::BishopPromo,
            9 => Flag::RookPromo,
            10 => Flag::QueenPromo,
            _ => panic!("invalid flag {}", value),
        }
    }
}

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
    pub fn flag(&self) -> Flag {
        Flag::from_u8((self.0 >> 12) as u8)
    }
    // next: convert text format to move (look to board's ep index decoding)
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut c: String = SQUARE_NAMES[self.start() as usize].to_owned();
        c += SQUARE_NAMES[self.end() as usize];
        c += match self.flag() {
            Flag::KnightPromo => "n",
            Flag::BishopPromo => "b",
            Flag::RookPromo => "r",
            Flag::QueenPromo => "q",
            _ => "",
        };
        write!(f, "{}", c)
    }
}