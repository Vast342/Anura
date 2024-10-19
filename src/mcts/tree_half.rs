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

use std::ops::{Index, IndexMut};

use crate::types::moves::Move;

use super::node::{Node, NodeIndex};

pub struct TreeHalf {
    nodes: Vec<Node>,
    length: u32,
    half: u32,
}
impl TreeHalf {
    pub fn new(size: usize, half_index: u32) -> Self {
        let mut half = Self {
            nodes: Vec::with_capacity(size),
            length: 0,
            half: half_index,
        };
        for _ in 0..size {
            half.nodes.push(Node::new(Move::NULL_MOVE, 0.0));
        }
        half
    }
    pub fn clear(&mut self) {
        self.length = 0;
    }
    pub fn len(&self) -> usize {
        self.length as usize
    }
    pub fn size(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_full(&self) -> bool {
        self.len() >= self.nodes.len()
    }
    pub fn push(&mut self, node: Node) -> NodeIndex {
        let new_index = NodeIndex::from_parts(self.length as usize, self.half as usize);
        self[new_index] = node;
        self.length += 1;
        new_index
    }
    pub fn clear_references(&self, _target_half: u32) {
        // uhhhhhhh
    }
}


impl Index<NodeIndex> for TreeHalf {
    type Output = Node;
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.nodes[index.index()]
    }
}

impl IndexMut<NodeIndex> for TreeHalf {
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.nodes[index.index()]
    }
}

