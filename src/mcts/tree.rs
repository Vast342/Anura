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
use super::node::Node;
use std::{
    mem,
    ops::{Index, IndexMut, Range},
};

pub const DEFAULT_TREE_SIZE: usize = 32;

pub struct SearchTree {
    tree: Vec<Node>,
}
impl Default for SearchTree {
    fn default() -> Self {
        Self::new(DEFAULT_TREE_SIZE)
    }
}
impl SearchTree {
    #[must_use]
    pub fn new(size_mb: usize) -> Self {
        let size_b = size_mb * 1024 * 1024;
        let size_entries = size_b / mem::size_of::<Node>();
        Self {
            tree: vec![Node::default(); size_entries],
        }
    }
    #[allow(clippy::len_without_is_empty)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.tree.len()
    }
    pub fn set_size(&mut self, new_size: usize) {
        let size_b = new_size * 1024 * 1024;
        let size_entries = size_b / mem::size_of::<Node>();
        self.tree = vec![Node::default(); size_entries];
    }
    pub fn reset(&mut self) {
        self.tree.clear();
        self.tree.shrink_to_fit();
    }
    pub fn push(&mut self, node: Node) {
        self.tree.push(node);
    }
}
impl Index<usize> for SearchTree {
    type Output = Node;
    fn index(&self, index: usize) -> &Self::Output {
        &self.tree[index]
    }
}
impl IndexMut<usize> for SearchTree {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.tree[index]
    }
}
impl Index<Range<usize>> for SearchTree {
    type Output = [Node];
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.tree[index]
    }
}
