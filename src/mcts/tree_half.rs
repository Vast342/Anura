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

use std::ops::{Index, IndexMut, Range};

use crate::types::moves::Move;

use super::node::Node;

/*
    Stuff to figure out:
    - external indexing (maybe keep track of which half it is on?)
    - maybe adjust the node internally so that the values are as they should be for indexing
    - when do nodes get copied over and which ones (i think it's as they're read but idk)
*/

pub struct TreeHalf {
    nodes: Vec<Node>,
    length: usize,
}
impl TreeHalf {
    pub fn new(size: usize) -> Self {
        let mut half = Self {
            nodes: Vec::with_capacity(size),
            length: 0,
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
        self.length
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_full(&self) -> bool {
        self.len() >= self.nodes.len()
    }

    pub fn push(&mut self, node: Node) -> Option<usize> {
        if self.is_full() {
            return None;
        }
        let len = self.length;
        self[len] = node;
        self.length += 1;
        Some(len)
    }
}

impl Index<usize> for TreeHalf {
    type Output = Node;
    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl IndexMut<usize> for TreeHalf {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

impl Index<Range<usize>> for TreeHalf {
    type Output = [Node];
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.nodes[index]
    }
}
