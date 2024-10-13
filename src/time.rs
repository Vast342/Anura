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
    const fn time_allocated(&self) -> u128 {
        self.time / 20 + self.increment / 2
    }
    pub fn check(&self, tim: u128, nodes: u128, depth: u32) -> bool {
        if self.use_time && tim >= self.time_allocated() {
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
