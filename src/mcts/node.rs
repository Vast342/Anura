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
use std::ops::Range;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum GameResult {
    Win,
    Draw,
    Loss,
    Ongoing,
}
impl GameResult {
    pub fn score(self) -> Option<f32> {
        match self {
            GameResult::Win => Some(1.0),
            GameResult::Draw => Some(0.5),
            GameResult::Loss => Some(0.0),
            GameResult::Ongoing => None,
        }
    }
    pub fn is_terminal(self) -> bool {
        self != Self::Ongoing
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Node {
    pub mov: Move,
    pub first_child: u32,
    pub child_count: u8,
    pub visits: u32,
    pub total_score: f32,
    pub result: GameResult,
    pub policy: f32,
    pub gini_impurity: f32,
}
impl Node {
    pub fn new(mov: Move, policy: f32) -> Self {
        Self {
            mov,
            first_child: 0,
            child_count: 0,
            visits: 0,
            total_score: 0.0,
            result: GameResult::Ongoing,
            policy,
            gini_impurity: 0.0,
        }
    }
    pub fn average_score(&self) -> f32 {
        self.total_score / self.visits as f32
    }
    pub fn children_range(&self) -> Range<usize> {
        let start = self.first_child as usize;
        let end = start + self.child_count as usize;
        start..end
    }
    pub fn dereference(&mut self) {
        self.first_child = 0;
        self.child_count = 0;
    }
}
