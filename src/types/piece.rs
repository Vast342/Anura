/*
    Anura
    Copyright (C) 2025 Joseph Pasfield

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

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Piece(pub u8);

#[derive(PartialEq, PartialOrd, Eq, Ord)]
#[repr(u8)]
pub enum Types {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    None,
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
#[repr(u8)]
pub enum Colors {
    Black,
    White,
    None,
}

impl Piece {
    #[must_use]
    pub const fn color(self) -> u8 {
        self.0 >> 3
    }
    #[must_use]
    pub const fn piece(self) -> u8 {
        self.0 & 0b0111
    }
    #[must_use]
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
        write!(f, "{c}")
    }
}
