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
use super::{node::Node, tree_half::TreeHalf};

pub const DEFAULT_HASH_SIZE: usize = 64;

pub struct SearchTree {
    halves: [TreeHalf; 2],
    current_half: usize,
    half_size: usize,
}
impl Default for SearchTree {
    fn default() -> Self {
        Self::new()
    }
}
impl SearchTree {
    pub fn new() -> Self {
        let size_mb = DEFAULT_HASH_SIZE / 2;
        let size_b = size_mb * 1024 * 1024;
        let size_entries = size_b / std::mem::size_of::<Node>();
        Self {
            halves: [TreeHalf::new(size_entries), TreeHalf::new(size_entries)],
            current_half: 0,
            half_size: size_entries,
        }
    }

    pub fn resize(&mut self, new_size: usize) {
        let size_mb = new_size / 2;
        let size_b = size_mb * 1024 * 1024;
        let size_entries = size_b / std::mem::size_of::<Node>();
        self.halves = [TreeHalf::new(size_entries), TreeHalf::new(size_entries)];
        self.current_half = 0;
        self.half_size = size_entries;
        self.reset();
    }

    pub fn next(&self) -> usize {
        self.current_half * self.half_size + self.halves[self.current_half].len()
    }

    pub fn reset(&mut self) {
        self.halves[0].clear();
        self.halves[1].clear();
    }

    pub fn push(&mut self, node: Node) {
        if self.halves[self.current_half].is_full() {
            println!("switching halves");
            // switch halves
            self.current_half = 1 - self.current_half;
            self.halves[self.current_half].clear();
            // ensure root node is first in the new entry
            self.halves[self.current_half].push(self.halves[1 - self.current_half][0]);
        }
        // push node to current half
        self.halves[self.current_half].push(node);
    }

    pub fn index(&mut self, index: usize) -> &Node {
        // if it's on previous half, copy it over and pass reference to the new one
        let half = index / self.half_size;
        let mut ind = index % self.half_size;
        if half != self.current_half {
            let mut node = self.halves[half][ind];
            node.dereference();
            ind = self.halves[self.current_half].len();
            self.halves[self.current_half].push(node);
        }

        &self.halves[self.current_half][ind]
    }

    pub fn index_mut(&mut self, index: usize) -> &mut Node {
        // if it's on previous half, copy it over and pass reference to the new one
        let half = index / self.half_size;
        let mut ind = index % self.half_size;
        if half != self.current_half {
            let mut node = self.halves[half][ind];
            node.dereference();
            ind = self.halves[self.current_half].len();
            self.halves[self.current_half].push(node);
        }
        &mut self.halves[self.current_half][ind]
    }
}
