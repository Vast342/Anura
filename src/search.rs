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

use crate::{board::Board, types::{moves::Move, MoveList}};
use std::time::Instant;
use std::ops::Range;

const MATE_SCORE: i16 = 32000;
const EVAL_SCALE: u16 = 400;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum GameResult {
    Win,
    Draw,
    Loss,
    Ongoing,
}

impl GameResult {
    fn floatify(self) -> Option<f32> {
        match self {
            GameResult::Win => Some(1.0),
            GameResult::Draw => Some(0.5),
            GameResult::Loss => Some(0.0),
            GameResult::Ongoing => None,
        }
    }

    fn is_terminal(self) -> bool {
        self != Self::Ongoing
    }
}

struct Node {
    parent: u32,
    mov: Move,
    first_child: u32,
    child_count: u8,
    visits: u32,
    total_score: f32,
    result: GameResult,
}

impl Node {
    fn new(parent: u32, mov: Move) -> Self {
        Self {
            parent,
            mov,
            first_child: 0,
            child_count: 0,
            visits: 0,
            total_score: 0.0,
            result: GameResult::Ongoing,
        }
    }

    fn avg(&self) -> f32 {
        self.total_score / self.visits as f32
    }

    fn children_range(&self) -> Range<usize> {
        let start = self.first_child as usize;
        let end = start + self.child_count as usize;
        start..end
    }
}

pub struct Engine {
    tree: Vec<Node>,
    board: Board,
    depth: u32,
    pub nodes: u128,
    start: Instant,
}

impl Engine {
    #[must_use] pub fn new() -> Self {
        Self{tree: vec!(), board: Board::new(), depth: 0, nodes: 0, start: Instant::now()}
    }
    fn select(&self) -> usize{

        0
    }

    fn expand(&mut self, node_idx: usize) {

    }

    // using my normal eval as a value net here so it actually just evaluates
    fn simulate(&self, node_idx: usize) -> f32 {
        let node = &self.tree[node_idx];
        node.result.floatify().unwrap_or_else(|| {
            1.0 / (1.0 + (-self.board.evaluate() as f32 / EVAL_SCALE as f32).exp())
        })
    }

    fn backprop(&mut self, mut node_idx: usize, mut result: f32) {

    }

    pub fn search(&mut self, board: Board, time: u128, info: bool) -> Move {
        self.nodes = 0;
        let mut seldepth = 0;
        let mut total_depth = 0;

        while !(self.start.elapsed().as_millis() > time / 20) {
            self.board = board.clone();
            self.depth = 1;

            // selection
            let node_idx = self.select();
            let node = &self.tree[node_idx];

            // expansion
            if !node.result.is_terminal() {
                self.expand(node_idx);
            }

            // simulation
            let result = self.simulate(node_idx);
            // backpropogation
            self.backprop(node_idx, result);


            total_depth += self.depth;
            if self.depth > seldepth {
                seldepth = self.depth;
                // info
                let duration = self.start.elapsed().as_millis();
                if info {
                    let nps = if duration == 0 {
                        0
                    } else {
                        self.nodes * 1000 / duration
                    };
                    println!("info depth {} nodes {} time {} nps {} score cp {}", self.depth, self.nodes, duration, nps, 0);
                }
            }

        }

        self.tree.clear();
        self.tree.shrink_to_fit();

        Move::new_unchecked(0, 0, 0)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}