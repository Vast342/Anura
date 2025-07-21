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

use crate::board::{Board, Position, SQUARE_NAMES};

use super::{
    piece::{Piece, Types},
    square::Square,
};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
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
    #[must_use]
    pub const fn new_unchecked(from: u8, to: u8, flag: u8) -> Self {
        //debug_assert!(from <= 63, "invalid from square {from}");
        //debug_assert!(to <= 63, "invalid to square {to}");
        Self(((flag as u16) << 12) | ((to as u16) << 6) | from as u16)
    }
    pub const NULL_MOVE: Self = Self::new_unchecked(0, 0, 0);
    #[must_use]
    pub const fn from(&self) -> u8 {
        (self.0 & 0b11_1111) as u8
    }
    #[must_use]
    pub const fn to(&self) -> u8 {
        ((self.0 >> 6) & 0b11_1111) as u8
    }
    #[must_use]
    pub fn flag(&self) -> Flag {
        Flag::from_u8((self.0 >> 12) as u8)
    }

    pub fn is_promotion(&self) -> bool {
        self.flag() >= Flag::KnightPromo
    }

    pub fn to_mf(&self, position: &Position) -> u16 {
        let current_flag = self.flag();
        let mut flag = match current_flag {
            Flag::DoublePush => 1,
            Flag::WKCastle | Flag::BKCastle => 2,
            Flag::WQCastle | Flag::BQCastle => 3,
            Flag::EnPassant => 5,
            Flag::KnightPromo => 8,
            Flag::BishopPromo => 9,
            Flag::RookPromo => 10,
            Flag::QueenPromo => 11,
            _ => 0,
        };
        // if capture add 4
        if position.piece_on_square(Square(self.to())) != Piece(6) {
            flag += 4;
        }
        ((self.from() as u16) << 10) | ((self.to() as u16) << 4) | flag
    }
    // next: convert text format to move (look to board's ep index decoding)
    #[must_use]
    pub fn from_text(text: &str, board: &Board) -> Self {
        let state = board.states.last().expect("teehee");
        let from_text = text[..2].to_owned();
        let to_text = text[2..4].to_owned();
        let mut from: u8 = 0;
        let mut to: u8 = 0;
        let mut flag: Flag = Flag::Normal;
        for i in 0..64 {
            if from_text == SQUARE_NAMES[i as usize] {
                from = i;
                break;
            }
        }
        for i in 0..64 {
            if to_text == SQUARE_NAMES[i as usize] {
                to = i;
                break;
            }
        }
        if text.chars().count() > 4 {
            flag = match text.chars().last().expect("what") {
                'n' => Flag::KnightPromo,
                'b' => Flag::BishopPromo,
                'r' => Flag::RookPromo,
                'q' => Flag::QueenPromo,
                _ => panic!("no promotion but length is over 4"),
            }
        } else {
            let castling = state.castling;
            if (castling & 1) != 0 && text == "e1g1" {
                flag = Flag::WKCastle;
            } else if (castling & 2) != 0 && text == "e1c1" {
                flag = Flag::WQCastle;
            } else if (castling & 4) != 0 && text == "e8g8" {
                flag = Flag::BKCastle;
            } else if (castling & 8) != 0 && text == "e8c8" {
                flag = Flag::BQCastle;
            } else if state.piece_on_square(Square(from)).piece() == Types::Pawn as u8
                && Square(to) == state.ep_index
            {
                flag = Flag::EnPassant;
            } else if state.piece_on_square(Square(from)).piece() == Types::Pawn as u8
                && from.abs_diff(to) == 16
            {
                flag = Flag::DoublePush;
            }
        }

        Self::new_unchecked(from, to, flag as u8)
    }
    pub fn to_other_string(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut c: String = SQUARE_NAMES[self.from() as usize].to_owned();
        c += SQUARE_NAMES[self.to() as usize];
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
