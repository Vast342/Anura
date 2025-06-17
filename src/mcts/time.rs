use crate::tunable::Tunables;

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
#[derive(Default, Clone, Copy)]
pub struct Limiters {
    use_time: bool,
    time: u128,
    increment: u128,
    use_nodes: bool,
    node_lim: u128,
    pub use_depth: bool,
    depth_limit: u32,
    use_move_time: bool,
    move_time: u128,
}

impl Limiters {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn load_values(&mut self, tim: u128, inc: u128, nodes: u128, depth: u32, movetime: u128) {
        self.use_time = tim != 0;
        self.use_nodes = nodes != 0;
        self.use_depth = depth != 0;
        self.use_move_time = movetime != 0;
        self.time = tim;
        self.increment = inc;
        self.node_lim = nodes;
        self.depth_limit = depth;
        self.move_time = movetime;
    }
    const fn time_allocated(&self, tunables: &Tunables) -> u128 {
        (self.time as f32 / tunables.time_divisor()) as u128
            + (self.increment as f32 / tunables.inc_divisor()) as u128
    }
    pub fn check(&self, tim: u128, nodes: u128, depth: u32, tunables: &Tunables) -> bool {
        if self.use_time && tim >= self.time_allocated(tunables) {
            return false;
        }
        if self.use_nodes && nodes >= self.node_lim {
            return false;
        }
        if self.use_depth && depth > self.depth_limit {
            return false;
        }
        if self.use_move_time && tim >= self.move_time {
            return false;
        }
        true
    }
}
