use std::ops::{Index, IndexMut, Range};

use super::node::Node;


pub struct SearchTree {
    tree: Vec<Node>,
}

impl Default for SearchTree {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchTree {
    pub fn new() -> Self {
        Self { tree: vec![] }
    }
    pub fn len(&self) -> usize {
        self.tree.len()
    }
    pub fn reset(&mut self) {
        self.tree.clear();
        self.tree.shrink_to_fit();
    }
    pub fn push(&mut self, node: Node) {
        self.tree.push(node)
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