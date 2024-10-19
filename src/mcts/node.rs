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
use crate::types::moves::Move;
use std::ops::{Add, Range};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum GameResult {
    Win,
    Draw,
    Loss,
    Ongoing,
}
impl GameResult {
    #[must_use]
    pub fn score(self, ctm: u8, root_ctm: u8) -> Option<f32> {
        match self {
            Self::Win => Some(1.0),
            Self::Draw => Some(0.02f32.mul_add(u32::from(ctm == root_ctm) as f32, 0.5 - 0.01)),
            Self::Loss => Some(0.0),
            Self::Ongoing => None,
        }
    }
    #[must_use]
    pub fn is_terminal(self) -> bool {
        self != Self::Ongoing
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Node {
    pub mov: Move,
    pub first_child: NodeIndex,
    pub child_count: u8,
    pub visits: u32,
    pub total_score: f32,
    pub result: GameResult,
    pub policy: f32,
}
impl Node {
    #[must_use]
    pub fn new(mov: Move, policy: f32) -> Self {
        Self {
            mov,
            first_child: NodeIndex::NULL,
            child_count: 0,
            visits: 0,
            total_score: 0.0,
            result: GameResult::Ongoing,
            policy,
        }
    }
    #[must_use]
    pub fn average_score(&self) -> f32 {
        self.total_score / self.visits as f32
    }
    #[must_use]
    pub fn children_range(&self) -> Range<usize> {
        let start = self.first_child.index();
        let end = start + self.child_count as usize;
        start..end
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new(Move::NULL_MOVE, 0.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeIndex(pub usize);
impl NodeIndex {
    pub const NULL: NodeIndex = Self(usize::MAX);
    pub fn from_raw(value: usize) -> Self {
        Self(value)
    }
    pub fn from_parts(index: usize, half: usize) -> Self {
        Self((half << 30) | (index & 0x3FFFFFFF))
    }
    pub fn get_raw(&self) -> usize {
        self.0
    }
    pub fn index(&self) -> usize {
        self.0 & 0x3FFFFFFF
    }
    pub fn half(&self) -> usize {
        (self.0 >> 30) as usize
    }
    pub fn is_null(&self) -> bool {
        *self == Self::NULL
    }
}

impl Add for NodeIndex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
