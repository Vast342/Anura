/*
    Anura
    Copyright (C) 2024 Joseph Pasfield

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
    fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::WKCastle,
            2 => Self::WQCastle,
            3 => Self::BKCastle,
            4 => Self::BQCastle,
            5 => Self::EnPassant,
            6 => Self::DoublePush,
            7 => Self::KnightPromo,
            8 => Self::BishopPromo,
            9 => Self::RookPromo,
            10 => Self::QueenPromo,
            _ => panic!("invalid flag {value}"),
        }
    }
}

impl Move {
    #[must_use] pub fn new_unchecked(start: u8, end: u8, flag: u8) -> Self {
        debug_assert!(start < 63, "invalid start square");
        debug_assert!(end < 63, "invalid end square");
        Self((u16::from(flag) << 12) | (u16::from(end) << 6) | u16::from(start))
    }
    #[must_use] pub const fn start(&self) -> u8 {
        (self.0 & 0b11_1111) as u8
    }
    #[must_use] pub const fn end(&self) -> u8 {
        ((self.0 >> 6) & 0b11_1111) as u8
    }
    #[must_use] pub fn flag(&self) -> Flag {
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
        write!(f, "{c}")
    }
}