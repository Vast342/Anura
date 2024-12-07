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
    mcts::time::Limiters,
    nets::policy::PolicyAccumulator,
    types::{moves::Move, MoveList},
    uci::UciOptions,
};
use std::time::Instant;

use super::{
    node::{GameResult, Node},
    tree::{SearchTree, IND_MASK},
};

const MATE_SCORE: i32 = 32000;
pub const EVAL_SCALE: u16 = 400;

pub struct SearchParams {
    pub cpuct: f32,
    pub fpu: f32,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            cpuct: std::f32::consts::SQRT_2,
            fpu: 0.5,
        }
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
    tree: SearchTree,
    board: Board,
    depth: u32,
    pub nodes: u128,
    start: Instant,
    root_ctm: u8,
    policy: PolicyAccumulator,
}

impl Engine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tree: SearchTree::default(),
            board: Board::new(),
            depth: 0,
            nodes: 0,
            start: Instant::now(),
            root_ctm: 1,
            policy: PolicyAccumulator::default(),
        }
    }
    fn select(&mut self, current: usize, params: &SearchParams) -> usize {
        let node = self.tree[current];

        
        #[cfg(feature = "datagen")]
        let e_scale = (node.visits as f32).sqrt();

        #[cfg(not(feature = "datagen"))]
        let e_scale = {
            let mut scale = (node.visits as f32).sqrt();
            // values from monty master, going to be tuned eventually:tm:
            scale *= (0.463 - 1.567 * (node.gini_impurity + 0.001).ln()).min(2.26);
            scale
        };
        
        let e = params.cpuct * e_scale;

        let mut best_child = 0;
        let mut best_child_uct = f32::NEG_INFINITY;
        for child_idx in node.children_range() {
            let child = self.tree[child_idx];
            let average_score = if child.visits == 0 {
                params.fpu
            } else {
                child.average_score()
            };
            let p = child.policy;
            let uct = average_score + e * p / (1 + child.visits) as f32;

            if uct > best_child_uct {
                best_child = child_idx;
                best_child_uct = uct;
            }
        }

        best_child
    }

    fn expand(&mut self, node_idx: usize, root: bool) -> Option<()> {
        let next = self.tree.next() as u32;
        let half_size = self.tree.half_size;
        let node = &mut self.tree[node_idx];

        if self.board.is_drawn() {
            node.result = GameResult::Draw;
            return Some(());
        }

        let mut moves = MoveList::new();
        self.board.get_moves(&mut moves);

        // checkmate or stalemate
        if moves.is_empty() {
            node.result = if self.board.in_check() {
                GameResult::Loss
            } else {
                GameResult::Draw
            };
            return Some(());
        }

        if (next as usize & IND_MASK) + moves.len() as usize >= half_size {
            return None;
        }

        // get initial policy values
        self.board.policy_load(&mut self.policy);
        let mut policy: Vec<f32> = vec![0.0; moves.len()];
        let mut policy_sum: f32 = 0.0;
        let mut sum_of_squares: f32 = 0.0;
        for i in 0..moves.len() {
            let unscaled = self.board.get_policy(moves[i], &mut self.policy);
            policy[i] = (unscaled
                / (1.0 + 2.5 * root as i32 as f32))
                .exp();
            policy_sum += policy[i];
        }
        // normalize
        for item in policy.iter_mut().take(moves.len()) {
            *item /= policy_sum;
            sum_of_squares += *item * *item;
        }

        node.first_child = next;
        node.child_count = moves.len() as u8;
        node.gini_impurity = (1.0 - sum_of_squares).clamp(0.0, 1.0);

        for i in 0..moves.len() {
            let node2 = Node::new(moves[i], policy[i]);
            self.tree.push(node2)?;
        }

        Some(())
    }

    // using my normal eval as a value net here so it actually just evaluates
    fn simulate(&mut self, node_idx: usize) -> f32 {
        let node = self.tree[node_idx];
        node.result
            .score(self.board.ctm, self.root_ctm)
            .unwrap_or_else(|| {
                1.0 / (1.0 + (-self.board.evaluate() as f32 / EVAL_SCALE as f32).exp())
            })
    }

    fn mcts(&mut self, current_node: usize, root: bool, params: &SearchParams) -> Option<f32> {
        let current_node_ref = &self.tree[current_node];

        let mut score =
            if !root && (current_node_ref.result.is_terminal() || current_node_ref.visits == 0) {
                self.simulate(current_node)
            } else {
                if current_node_ref.child_count == 0 {
                    self.expand(current_node, root)?;
                    if self.tree[current_node].result.is_terminal() {
                        return Some(self.simulate(current_node));
                    }
                }

                self.tree.copy_children(current_node)?;

                let next_index = self.select(current_node, params);

                self.board.make_move(self.tree[next_index].mov);
                self.depth += 1;

                self.mcts(next_index, false, params)?
            };

        score = 1.0 - score;

        let current_node_ref_mut = &mut self.tree[current_node];
        current_node_ref_mut.visits += 1;
        current_node_ref_mut.total_score += score;

        Some(score)
    }

    fn get_best_move(&mut self, root_node: usize) -> (usize, f32) {
        let root = self.tree[root_node];

        let mut best = None;
        let mut best_score = f32::NEG_INFINITY;

        for node_idx in root.children_range() {
            let node = self.tree[node_idx];
            let score = node.average_score();

            if score > best_score {
                best = Some(node_idx);
                best_score = score;
            }
        }

        (best.expect("nothing"), best_score)
    }

    pub fn get_pv(&mut self, root_node: usize) -> (Vec<Move>, f32, bool) {
        let (root_best_child, root_best_score) = self.get_best_move(root_node);
        let mut pv = vec![];
        pv.push(self.tree[root_best_child].mov);
        let mut ends_in_mate = false;

        let mut node_idx = root_best_child;
        loop {
            let node = self.tree[node_idx];
            if node.result.is_terminal() || node.child_count == 0 {
                if node.result == GameResult::Loss || node.result == GameResult::Win {
                    ends_in_mate = true;
                }
                break;
            }
            let mut has_valid_child = false;
            let mut best_child_idx = 0;
            let mut best_child_score = f32::NEG_INFINITY;
            for child_idx in node.children_range() {
                let child = self.tree[child_idx];
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

        (pv, root_best_score, ends_in_mate)
    }

    // todo 1: Tree Reuse
    // todo 2: SMP
    // todo 3: Better value net
    // todo 4: Actual policy net
    pub fn search(
        &mut self,
        board: Board,
        limiters: Limiters,
        info: bool,
        options: &UciOptions,
    ) -> Move {
        self.nodes = 0;
        let mut seldepth = 0;
        let mut total_depth: usize = 0;
        let mut prev_avg_depth = 1;
        let mut avg_depth = 0;
        self.start = Instant::now();
        let mut last_print = Instant::now();

        self.tree.push(Node::new(Move::NULL_MOVE, 0.0));

        let root_state = board.states.last().expect("bruh you gave an empty board");
        let root_ctm = board.ctm;
        self.root_ctm = root_ctm;
        let params = SearchParams::default();

        while limiters.check(self.start.elapsed().as_millis(), self.nodes, avg_depth) {
            self.board.load_state(root_state, root_ctm);
            self.depth = 1;

            let result = self.mcts(self.tree.root_node(), true, &params);

            self.nodes += 1;
            total_depth += self.depth as usize;

            if self.depth > seldepth {
                seldepth = self.depth;
            }

            // info 
            avg_depth = (total_depth as f64 / self.nodes as f64).round() as u32;
            if avg_depth > prev_avg_depth || last_print.elapsed().as_secs_f32() > 1.0 {
                let duration = self.start.elapsed().as_millis();
                if info {
                    self.print_info(
                        self.tree.root_node(),
                        avg_depth - 1,
                        seldepth,
                        duration,
                        false,
                        options,
                    );
                }
                prev_avg_depth = avg_depth;
                last_print = Instant::now();
            }

            if result == None {
                self.tree.switch_halves();
            }
        }
        if !limiters.use_depth {
            let duration = self.start.elapsed().as_millis();
            avg_depth = (total_depth as f64 / self.nodes as f64).round() as u32 - 1;
            if info {
                self.print_info(
                    self.tree.root_node(),
                    avg_depth,
                    seldepth,
                    duration,
                    true,
                    options,
                );
            }
        }

        let (index, _best_score) = self.get_best_move(self.tree.root_node());
        let best_move = self.tree[index].mov;

        self.tree.reset();

        best_move
    }
    #[cfg(feature = "datagen")]
    pub fn datagen_search(&mut self, board: Board) -> (Move, i32, Vec<(Move, u16)>) {
        self.nodes = 0;
        self.start = Instant::now();

        self.tree.push(Node::new(Move::NULL_MOVE, 0.0));

        let root_state = board.states.last().expect("bruh you gave an empty board");
        let root_ctm = board.ctm;
        let root_node = 0;
        let params = SearchParams::default();

        while self.nodes < NODE_LIMIT {
            self.board.load_state(root_state, root_ctm);

            self.mcts(root_node, true, &params);

            self.nodes += 1;
        }

        let (best_node_idx, best_score) = self.get_best_move(root_node);
        let best_move = self.tree[best_node_idx].mov;

        // get visit distribution
        let root_node = self.tree[0];
        let mut visit_points: Vec<(Move, u16)> = vec![];
        for child_idx in root_node.children_range() {
            let child_node = self.tree[child_idx];
            visit_points.push((child_node.mov, child_node.visits as u16));
        }

        self.tree.reset();
        (best_move, to_cp(best_score), visit_points)
    }
    fn print_info(
        &mut self,
        root_node: usize,
        depth: u32,
        seldepth: u32,
        duration: u128,
        final_info: bool,
        options: &UciOptions,
    ) {
        if final_info && options.more_info {
            // potential todo: even more information
            for node_idx in self.tree[0].children_range() {
                let this_node = self.tree[node_idx];
                let score = this_node.average_score();

                println!(
                    "{}: visits: {}, average score: {}",
                    this_node.mov, this_node.visits, score,
                );
            }
        }
        let (pv, score, ends_in_mate) = self.get_pv(root_node);
        let nps = if duration == 0 {
            0
        } else {
            self.nodes * 1000 / duration
        };
        print!(
            "info depth {} seldepth {} nodes {} time {} nps {} ",
            depth, seldepth, self.nodes, duration, nps,
        );
        if ends_in_mate {
            print!("score mate {} ", pv.len() / 2);
        } else {
            print!("score cp {} ", to_cp(score));
        }
        print!("pv");
        for mov in &pv {
            print!(" {}", mov);
        }
        println!();
    }
    pub fn resize(&mut self, new_size: usize) {
        self.tree.resize(new_size);
    }
    pub fn new_game(&mut self) {
        self.tree.reset();
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
