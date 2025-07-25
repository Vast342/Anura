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
#[cfg(feature = "datagen")]
use crate::datagen::NODE_LIMIT;
use crate::{
    board::{Board, Position},
    mcts::time::Limiters,
    nets::policy::PolicyAccumulator,
    tunable::Tunables,
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
    policy: PolicyAccumulator,
}

impl Engine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tree: SearchTree::default(),
            board: Board::default(),
            depth: 0,
            nodes: 0,
            start: Instant::now(),
            policy: PolicyAccumulator::default(),
        }
    }
    fn select(&mut self, current: usize, tunables: &Tunables, root: bool) -> usize {
        let node = self.tree[current];

        #[cfg(feature = "datagen")]
        let e_scale = (node.visits as f32).sqrt();

        #[cfg(not(feature = "datagen"))]
        let e_scale = {
            let mut scale = (node.visits as f32).sqrt();
            scale *= (tunables.gini_base()
                - tunables.gini_log_mult() * (node.gini_impurity + 0.001).ln())
            .min(tunables.gini_min());
            scale
        };

        let mut cpuct = if root {
            tunables.root_cpuct()
        } else {
            tunables.default_cpuct()
        };
        let vis_scale = tunables.cpuct_visits_scale() * 128.0;
        cpuct *= 1.0 + ((node.visits as f32 + vis_scale) / vis_scale).ln();

        let e = cpuct * e_scale;

        let parent_q = node.average_score();

        let mut best_child = 0;
        let mut best_child_uct = f32::NEG_INFINITY;
        for child_idx in node.children_range() {
            let child = self.tree[child_idx];
            let average_score = if child.visits == 0 {
                1.0 - parent_q
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

    fn expand(&mut self, node_idx: usize, root: bool, tunables: &Tunables) -> Option<()> {
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

        if (next as usize & IND_MASK) + moves.len() >= half_size {
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
                / (tunables.default_pst() + tunables.root_pst_bonus() * root as i32 as f32))
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

    // not an actual simulation, but for nomenclature consistent with normal mcts, i decided to call it that.
    fn simulate(&mut self, node_idx: usize) -> f32 {
        let node = self.tree[node_idx];
        node.result.score().unwrap_or_else(|| {
            1.0 / (1.0 + (-self.board.evaluate() as f32 / EVAL_SCALE as f32).exp())
        })
    }

    fn mcts(&mut self, current_node: usize, root: bool, tunables: &Tunables) -> Option<f32> {
        let current_node_ref = &self.tree[current_node];

        let mut score = if current_node_ref.result.is_terminal() || current_node_ref.visits == 0 {
            self.simulate(current_node)
        } else {
            if current_node_ref.child_count == 0 {
                self.expand(current_node, root, tunables)?;
                if self.tree[current_node].result.is_terminal() {
                    return Some(self.simulate(current_node));
                }
            }

            self.tree.copy_children(current_node)?;

            let next_index = self.select(current_node, tunables, root);

            self.board.make_move(self.tree[next_index].mov);
            self.depth += 1;

            self.mcts(next_index, false, tunables)?
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
        let mut pv = vec![];
        let mut ends_in_mate = false;
        let mut root_score = 0.0;
        let mut node_idx = root_node;
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
            if node_idx == root_node {
                root_score = best_child_score;
            }
            node_idx = best_child_idx;
        }

        (pv, root_score, ends_in_mate)
    }

    pub fn find(&mut self, start: usize, state: &Position, depth: u8) -> usize {
        let start_node = self.tree[start];

        if self.board.current_state() == state {
            return start;
        }
        if start == (1 << 31) - 1 || depth == 0 {
            return (1 << 31) - 1;
        }

        //let start_node = self.tree[start];
        //dbg!(start_node);

        for i in start_node.children_range() {
            let i_node = self.tree[i];
            self.board.make_move(i_node.mov);
            let found = self.find(i, state, depth - 1);
            self.board.undo_move();

            if found != (1 << 31) - 1 {
                return found;
            }
        }

        (1 << 31) - 1
    }

    // todo 1: Tree Reuse
    // todo 2: SMP
    pub fn search(
        &mut self,
        board: Board,
        limiters: Limiters,
        info: bool,
        options: &UciOptions,
        tunables: &Tunables,
    ) -> Move {
        self.nodes = 0;
        let mut seldepth = 0;
        let mut total_depth: usize = 0;
        let mut prev_avg_depth = 1;
        let mut avg_depth = 0;
        self.start = Instant::now();
        let mut last_print = Instant::now();

        let root_state = board.states.last().expect("bruh you gave an empty board");
        let root_ctm = board.ctm;
        // attempt to reuse tree
        if self.tree.is_empty() {
            self.tree.push(Node::new(Move::NULL_MOVE, 0.0));
        } else {
            let root = self.tree.root_node();
            let found = self.find(root, root_state, 2);
            if found != (1 << 31) - 1 && self.tree[found].child_count != 0 {
                self.tree[root] = self.tree[found];
            } else {
                self.tree.reset();
                self.tree.push(Node::new(Move::NULL_MOVE, 0.0));
            }
        };

        while limiters.check(
            self.start.elapsed().as_millis(),
            self.nodes,
            avg_depth,
            tunables,
        ) {
            self.board.load_state(root_state, root_ctm);
            self.depth = 1;

            let result = self.mcts(self.tree.root_node(), true, tunables);

            self.nodes += 1;
            total_depth += self.depth as usize;

            if self.depth > seldepth {
                seldepth = self.depth;
            }

            // info
            avg_depth = (total_depth as f64 / self.nodes as f64).round() as u32;
            if avg_depth > prev_avg_depth || last_print.elapsed().as_secs_f32() > 3.0 {
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

            if result.is_none() {
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

        self.board.load_state(root_state, root_ctm);

        best_move
    }
    #[cfg(feature = "datagen")]
    pub fn datagen_search(
        &mut self,
        board: Board,
        params: &Tunables,
    ) -> (Move, i32, Vec<(Move, u16)>) {
        self.nodes = 0;
        self.start = Instant::now();

        let root_state = board.states.last().expect("bruh you gave an empty board");
        let root_ctm = board.ctm;

        // attempt to reuse tree
        if self.tree.is_empty() {
            self.tree.push(Node::new(Move::NULL_MOVE, 0.0));
        } else {
            let root = self.tree.root_node();
            let found = self.find(root, root_state, 2);
            if found != (1 << 31) - 1 && self.tree[found].child_count != 0 {
                self.tree[root] = self.tree[found];
            } else {
                self.tree.reset();
                self.tree.push(Node::new(Move::NULL_MOVE, 0.0));
            }
        };

        while self.nodes < NODE_LIMIT {
            self.board.load_state(root_state, root_ctm);

            self.mcts(self.tree.root_node(), true, &params);

            self.nodes += 1;
        }

        let (best_node_idx, best_score) = self.get_best_move(self.tree.root_node());
        let best_move = self.tree[best_node_idx].mov;

        // get visit distribution
        let root_node = self.tree[self.tree.root_node()];
        let mut visit_points: Vec<(Move, u16)> = vec![];
        for child_idx in root_node.children_range() {
            let child_node = self.tree[child_idx];
            visit_points.push((child_node.mov, child_node.visits as u16));
        }

        self.board.load_state(root_state, root_ctm);
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
            let mut results = Vec::new();
            for node_idx in self.tree[0].children_range() {
                let this_node = self.tree[node_idx];
                let score = this_node.average_score();
                let pv = self.get_pv(node_idx).0;
                results.push((this_node.mov, this_node.visits, score, pv));
            }
            results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            for (mov, visits, score, pv) in results {
                print!(
                    "{:<6} visits: {:>8} | average score: {:>4} cp | pv",
                    format!("{}:", mov),
                    visits,
                    to_cp(score),
                );
                for pv_move in &pv {
                    print!(" {pv_move}");
                }
                println!();
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
            print!(" {mov}");
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
