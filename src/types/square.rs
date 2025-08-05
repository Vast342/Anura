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
use std::ops::AddAssign;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square(pub u8);

impl Square {
    pub const INVALID: Self = Self(64);

    pub fn flip(&mut self) {
        self.0 ^= 56;
    }

    pub fn flip_file(&mut self) {
        self.0 ^= 7;
    }

    #[must_use]
    pub const fn rank(&self) -> u8 /* i refuse to write a rank wrapper */ {
        self.0 / 8
    }

    #[must_use]
    pub const fn file(&self) -> u8 /* i refuse to write a file wrapper */ {
        self.0 & 0b111
    }

    #[must_use]
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    #[must_use]
    pub const fn from_rf(rank: u8, file: u8) -> Self {
        Self(rank * 8 + file)
    }
}

impl AddAssign for Square {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
