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
use std::ops::{Index, IndexMut, Range};

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
        Self {
            halves: [
                TreeHalf::new(DEFAULT_HASH_SIZE / 2),
                TreeHalf::new(DEFAULT_HASH_SIZE / 2),
            ],
            current_half: 0,
            half_size: DEFAULT_HASH_SIZE / 2,
        }
    }
    #[allow(clippy::len_without_is_empty)]
    pub fn next(&self) -> usize {
        self.current_half * self.half_size + self.halves[self.current_half].len()
    }
    pub fn reset(&mut self) {
        self.halves[0].clear();
        self.halves[1].clear();
    }
    pub fn push(&mut self, node: Node) {
        if self.halves[self.current_half].is_full() {
            // switch halves
            self.current_half = 1 - self.current_half;
            self.halves[self.current_half].clear();
        }
        // push node to current half
        self.halves[self.current_half].push(node);
    }
}
impl Index<usize> for SearchTree {
    type Output = Node;
    fn index(&self, index: usize) -> &Self::Output {
        // if it's on previous half, copy it over and pass reference to the new one
    }
}
impl IndexMut<usize> for SearchTree {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        //&mut self.halves[index]
    }
}
impl Index<Range<usize>> for SearchTree {
    type Output = [Node];
    fn index(&self, index: Range<usize>) -> &Self::Output {
        // maybe copy all of these over? idk
        //&self.halves[index]
    }
}
