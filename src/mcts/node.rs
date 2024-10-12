use std::ops::Range;

use crate::types::moves::Move;


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum GameResult {
    Win,
    Draw,
    Loss,
    Ongoing,
}

impl GameResult {
    pub fn score(self, ctm: u8, root_ctm: u8) -> Option<f32> {
        match self {
            GameResult::Win => Some(1.0),
            GameResult::Draw => Some(0.5 - 0.01 + 0.02 * (ctm == root_ctm) as u32 as f32),
            GameResult::Loss => Some(0.0),
            GameResult::Ongoing => None,
        }
    }

    pub fn is_terminal(self) -> bool {
        self != Self::Ongoing
    }
}

pub struct Node {
    pub mov: Move,
    pub first_child: u32,
    pub child_count: u8,
    pub visits: u32,
    pub total_score: f32,
    pub result: GameResult,
    pub policy: f32,
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
}