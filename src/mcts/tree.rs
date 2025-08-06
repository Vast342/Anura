use std::ops::{Index, IndexMut};

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
use super::{node::Node, tree_half::TreeHalf};

pub const DEFAULT_HASH_SIZE: usize = 64;

pub struct SearchTree {
    halves: [TreeHalf; 2],
    current_half: usize,
    pub half_size: usize,
}

impl Default for SearchTree {
    fn default() -> Self {
        Self::new()
    }
}

pub const IND_MASK: usize = 0x7fffffff;
impl SearchTree {
    pub fn new() -> Self {
        let size_mb = DEFAULT_HASH_SIZE;
        let size_b = size_mb * 1024 * 1024;
        let size_entries = (size_b / std::mem::size_of::<Node>()) / 2;
        Self {
            halves: [TreeHalf::new(size_entries), TreeHalf::new(size_entries)],
            current_half: 0,
            half_size: size_entries,
        }
    }

    pub fn resize(&mut self, new_size: usize) {
        let size_mb = new_size;
        let size_b = size_mb * 1024 * 1024;
        let size_entries = (size_b / std::mem::size_of::<Node>()) / 2;
        self.halves = [TreeHalf::new(size_entries), TreeHalf::new(size_entries)];
        self.current_half = 0;
        self.half_size = size_entries;
        self.reset();
    }

    pub fn root_node(&self) -> usize {
        self.current_half << 31
    }

    pub fn is_empty(&self) -> bool {
        self.halves[0].is_empty() && self.halves[1].is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.halves[self.current_half].is_full()
    }

    pub fn next(&self) -> usize {
        (self.current_half << 31) | self.halves[self.current_half].len()
    }

    pub fn reset(&mut self) {
        self.halves[0].clear();
        self.halves[1].clear();
        self.current_half = 0;
    }

    pub fn push(&mut self, node: Node) -> Option<()> {
        // push node to current half
        self.halves[self.current_half].push(node)?;
        Some(())
    }

    pub fn copy_children(&mut self, parent: usize) -> Option<()> {
        let parent_ind = parent & IND_MASK;
        let parent_node = self.halves[self.current_half][parent_ind];
        let child = parent_node.first_child as usize;
        let child_half = child >> 31;

        if child_half == self.current_half {
            return Some(());
        }

        let child_count = parent_node.child_count as usize;

        for this_child in child..(child + child_count) {
            let this_child_ind = this_child & IND_MASK;
            // this_child_ind ends up overflowing
            let this_child_node = self.halves[child_half][this_child_ind];
            self.halves[self.current_half].push(this_child_node)?;
        }
        self[parent].first_child = self.next() as u32 - child_count as u32;
        Some(())
    }

    pub fn switch_halves(&mut self) {
        // switch halves
        self.current_half = 1 - self.current_half;
        self.halves[self.current_half].clear();
        self.dereference();
        // ensure root node is first in the new entry
        self.halves[self.current_half].push(self.halves[1 - self.current_half][0]);
    }

    pub fn dereference(&mut self) {
        for i in 0..self.half_size {
            if self.halves[1 - self.current_half][i].first_child >> 31 == self.current_half as u32 {
                self.halves[1 - self.current_half][i].first_child = IND_MASK as u32;
                self.halves[1 - self.current_half][i].child_count = 0;
            }
        }
    }

    pub fn hashfull(&self) -> u16 {
        ((self.halves[self.current_half].len() as f32
            / self.halves[self.current_half].size() as f32)
            * 1000.0) as u16
    }
}

impl Index<usize> for SearchTree {
    type Output = Node;
    fn index(&self, index: usize) -> &Self::Output {
        let half = index >> 31;
        let ind = index & IND_MASK;
        &self.halves[half][ind]
    }
}

impl IndexMut<usize> for SearchTree {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let half = index >> 31;
        let ind = index & IND_MASK;
        &mut self.halves[half][ind]
    }
}
