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
#[cfg(feature = "datagen")]
use crate::datagen::NODE_LIMIT;
use crate::{
    board::Board,
    types::{moves::Move, MoveList},
    uci::UciOptions,
};
use std::ops::Range;
use std::time::Instant;

const MATE_SCORE: i32 = 32000;
pub const EVAL_SCALE: u16 = 400;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum GameResult {
    #[allow(unused)]
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
    policy: f32,
}

impl Node {
    fn new(parent: u32, mov: Move, policy: f32) -> Self {
        Self {
            parent,
            mov,
            first_child: 0,
            child_count: 0,
            visits: 0,
            total_score: 0.0,
            result: GameResult::Ongoing,
            policy,
        }
    }

    fn average_score(&self) -> f32 {
        self.total_score / self.visits as f32
    }

    fn children_range(&self) -> Range<usize> {
        let start = self.first_child as usize;
        let end = start + self.child_count as usize;
        start..end
    }
}

pub fn to_cp(score: f32) -> i32 {
    if score == 1.0 {
        MATE_SCORE
    } else if score == 0.0 {
        -MATE_SCORE
    } else {
        (-(EVAL_SCALE as f32) * (1.0 / score - 1.0).ln()) as i32
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
    #[must_use]
    pub fn new() -> Self {
        Self {
            tree: vec![],
            board: Board::new(),
            depth: 0,
            nodes: 0,
            start: Instant::now(),
        }
    }
    fn select(&mut self) -> usize {
        let mut current = 0;
        loop {
            let node = &self.tree[current as usize];
            if current != 0 {
                self.board.make_move(node.mov);
                self.depth += 1;
            }
            if node.result.is_terminal() || node.child_count == 0 {
                break;
            }

            let e = std::f32::consts::SQRT_2 * (node.visits as f32).sqrt();

            let mut best_child = None;
            let mut best_child_uct = f32::NEG_INFINITY;

            for (child_idx, child) in self.tree[node.children_range()].iter().enumerate() {
                let average_score = if child.visits == 0 {
                    0.5
                } else {
                    child.average_score()
                };
                let p = child.policy;
                let uct = average_score + e * p / (1 + child.visits) as f32;

                if uct > best_child_uct {
                    best_child = Some(child_idx);
                    best_child_uct = uct;
                }
            }

            current = (node.first_child as usize + best_child.unwrap()) as u32;
        }
        current as usize
    }

    fn expand(&mut self, node_idx: usize) {
        let next = self.tree.len() as u32;
        let node = &mut self.tree[node_idx];

        if self.board.is_drawn() {
            node.result = GameResult::Draw;
            return;
        }

        let mut moves = MoveList::new();
        self.board.get_moves(&mut moves);

        // get initial policy values
        let mut policy: Vec<f32> = vec![0.0; moves.len()];
        let mut policy_sum: f32 = 0.0;
        for i in 0..moves.len() {
            policy[i] = self.board.get_policy(moves[i]).exp();
            policy_sum += policy[i];
        }
        // normalize
        for i in 0..moves.len() {
            policy[i] = policy[i] / policy_sum;
        }

        // checkmate or stalemate
        if moves.is_empty() {
            node.result = if self.board.in_check() {
                GameResult::Loss
            } else {
                GameResult::Draw
            };
            return;
        }

        node.first_child = next;
        node.child_count = moves.len() as u8;

        for i in 0..moves.len() {
            let node = Node::new(node_idx as u32, moves[i], policy[i]);
            self.tree.push(node);
        }
    }

    // using my normal eval as a value net here so it actually just evaluates
    fn simulate(&self, node_idx: usize) -> f32 {
        let node = &self.tree[node_idx];
        node.result.floatify().unwrap_or_else(|| {
            1.0 / (1.0 + (-self.board.evaluate() as f32 / EVAL_SCALE as f32).exp())
        })
    }

    fn backprop(&mut self, mut node_idx: usize, mut result: f32) {
        loop {
            let node = &mut self.tree[node_idx];

            node.visits += 1;

            if node_idx == 0 {
                break;
            }
            // idea
            // result = 1.0 - 0.95 * result;
            result = 1.0 - result;
            node.total_score += result;

            node_idx = node.parent as usize;
        }
    }

    fn get_best_move(&self) -> (usize, f32) {
        let root = &self.tree[0];

        let mut best = None;
        let mut best_visits = 0u32;
        let mut best_score = f32::NEG_INFINITY;

        for node_idx in root.children_range() {
            let node = &self.tree[node_idx];
            let score = node.average_score();

            if node.visits > best_visits {
                best = Some(node_idx);
                best_visits = node.visits;
                best_score = score;
            }
        }

        (best.expect("nothing"), best_score)
    }

    pub fn get_pv(&mut self) -> (Vec<Move>, f32) {
        let (root_best_child, root_best_score) = self.get_best_move();
        let mut pv = vec![];
        pv.push(self.tree[root_best_child].mov);

        let mut node_idx = root_best_child;
        loop {
            let node = &self.tree[node_idx];
            if node.result.is_terminal() || node.child_count == 0 {
                break;
            }
            let mut has_valid_child = false;
            let mut best_child_idx = 0;
            let mut best_child_score = f32::NEG_INFINITY;
            for child_idx in node.children_range() {
                let child = &self.tree[child_idx];
                if child.visits == 0 {
                    continue;
                }
                has_valid_child = true;
                if child.average_score() > best_child_score {
                    best_child_idx = child_idx;
                    best_child_score = child.average_score();
                }
            }

            if !has_valid_child {
                break;
            }
            pv.push(self.tree[best_child_idx].mov);
            node_idx = best_child_idx;
        }

        (pv, root_best_score)
    }

    pub fn search(
        &mut self,
        board: Board,
        node_lim: u128,
        time: u128,
        inc: u128,
        depth_limit: u32,
        info: bool,
        options: &UciOptions,
    ) -> Move {
        self.nodes = 0;
        let mut seldepth = 0;
        let mut total_depth: usize = 0;
        let mut prev_avg_depth = 1;
        self.start = Instant::now();

        self.tree.push(Node::new(0, Move::NULL_MOVE, 0.0));

        let root_state = board.states.last().expect("bruh you gave an empty board");
        let root_ctm = board.ctm;

        while self.start.elapsed().as_millis() <= time / 20 + inc / 2 {
            self.board.load_state(root_state, root_ctm);
            self.depth = 1;

            // selection
            let node_idx = self.select();
            let node = &self.tree[node_idx];

            // expansion
            if !node.result.is_terminal() && node.visits != 0 {
                self.expand(node_idx);
            }

            // simulation
            let result = self.simulate(node_idx);
            // backpropogation
            self.backprop(node_idx, result);

            self.nodes += 1;
            if self.nodes > node_lim {
                break;
            }
            total_depth += self.depth as usize;

            if self.depth > seldepth {
                seldepth = self.depth;
            }

            // info
            let avg_depth = (total_depth as f64 / self.nodes as f64).round() as u32;
            if avg_depth >= depth_limit {
                break;
            }
            if avg_depth > prev_avg_depth {
                let duration = self.start.elapsed().as_millis();
                if info {
                    let (pv, score) = self.get_pv();
                    let nps = if duration == 0 {
                        0
                    } else {
                        self.nodes * 1000 / duration
                    };
                    print!(
                        "info depth {} seldepth {} nodes {} time {} nps {} score cp {} pv",
                        avg_depth - 1,
                        seldepth,
                        self.nodes,
                        duration,
                        nps,
                        to_cp(score)
                    );
                    for mov in &pv {
                        print!(" {}", mov.to_string());
                    }
                    println!();
                }
                prev_avg_depth = avg_depth;
            }
        }

        let (pv, best_score) = self.get_pv();

        let duration = self.start.elapsed().as_millis();
        let avg_depth = (total_depth as f64 / self.nodes as f64).round() as u32 - 1;
        if info {
            // todo: optional full breakdown of visit and score distribution, like voidstar's
            if options.more_info {
                for node_idx in self.tree[0].children_range() {
                    let this_node = &self.tree[node_idx];
                    let score = this_node.average_score();

                    println!(
                        "{}: visits: {}, average score: {}",
                        this_node.mov.to_string(),
                        this_node.visits,
                        score,
                    );
                }
            }
            let nps = if duration == 0 {
                0
            } else {
                self.nodes * 1000 / duration
            };
            print!(
                "info depth {} seldepth {} nodes {} time {} nps {} score cp {} pv",
                avg_depth,
                seldepth,
                self.nodes,
                duration,
                nps,
                to_cp(best_score),
            );
            for mov in &pv {
                print!(" {}", mov.to_string());
            }
            println!();
        }

        self.tree.clear();
        self.tree.shrink_to_fit();

        pv[0]
    }
    #[cfg(feature = "datagen")]
    pub fn datagen_search(&mut self, board: Board) -> (Move, i32, Vec<(Move, u16)>) {
        self.nodes = 0;
        self.start = Instant::now();

        self.tree.push(Node::new(0, Move::NULL_MOVE, 0.0));

        let root_state = board.states.last().expect("bruh you gave an empty board");
        let root_ctm = board.ctm;

        while self.nodes < NODE_LIMIT {
            self.board.load_state(root_state, root_ctm);

            // selection
            let node_idx = self.select();
            let node = &self.tree[node_idx];

            // expansion
            if !node.result.is_terminal() && node.visits != 0 {
                self.expand(node_idx);
            }

            // simulation
            let result = self.simulate(node_idx);
            // backpropogation
            self.backprop(node_idx, result);

            self.nodes += 1;
        }

        let (best_node_idx, best_score) = self.get_best_move();
        let best_move = self.tree[best_node_idx].mov;

        // get visit distribution
        let root_node = &self.tree[0];
        let mut visit_points: Vec<(Move, u16)> = vec![];
        for child_idx in root_node.children_range() {
            let child_node = &self.tree[child_idx];
            visit_points.push((child_node.mov, child_node.visits as u16));
        }

        self.tree.clear();
        self.tree.shrink_to_fit();

        (best_move, to_cp(best_score), visit_points)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
