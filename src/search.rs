use std::time::Instant;

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

pub struct Engine {
    nodes: u128,
    root_best_move: Move,
    start: Instant,
    hard_limit: u128,
    time_out: bool,
}

impl Engine {
    pub fn new() -> Self {
        Self{nodes: 0, root_best_move: Move::new_unchecked(0, 0, 0), start: Instant::now(), hard_limit: 0, time_out: false}
    }
    pub fn iteratively_deepen(&mut self, mut board: Board, time: u128) -> Move {
        self.nodes = 0;
        self.root_best_move = Move::new_unchecked(0, 0, 0);
        let mut prev_best: Move = self.root_best_move;
        self.start = Instant::now();
        self.hard_limit = time / 10;
        self.time_out = false;

        for depth in 1..100 {
            let score = self.negamax(&mut board, depth, 0);
            let duration = self.start.elapsed().as_millis();
            let nps = if duration == 0 {
                0
            } else {
                self.nodes * 1000 / duration
            };
            println!("info depth {} nodes {} time {} nps {} score cp {} pv {}", depth, self.nodes, duration, nps, score, self.root_best_move);
            if self.time_out {
                self.root_best_move = prev_best;
                break
            }

            if duration >= time / 30 {
                break
            }
            prev_best = self.root_best_move;
        }
        self.root_best_move
    }
    pub fn negamax(&mut self, board: &mut Board, depth: u8, ply: u8) -> i16 {
        if depth <= 0 { return board.evaluate() }
        if self.nodes % 4096 == 0 && (self.time_out || self.start.elapsed().as_millis() >= self.hard_limit) { 
            self.time_out = true;
            return 0 
        }

        let mut list: MoveList = MoveList::new();
        board.get_moves(&mut list);
        let mut best_score: i16 = -32000;
        let mut legal_moves = 0;
        for mov in list {
            if !board.make_move(mov) { continue; }
            legal_moves += 1;
            self.nodes += 1;

            let score = -self.negamax(board, depth - 1, ply + 1);
            
            if self.time_out { return 0 }

            board.undo_move();

            if score >= best_score {
                if ply == 0 { self.root_best_move = mov }
                best_score = score;
            }
        }
        if legal_moves == 0 {
            if board.in_check() {
                return -32000 + ply as i16
            } else {
                return 0
            }
        }
        best_score
    }
}