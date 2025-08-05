use crate::tunable::Tunables;

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
    #[cfg(feature = "datagen")]
    min_kld: f64,
    #[cfg(feature = "datagen")]
    use_kld: bool,
}

impl Limiters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_values(
        &mut self,
        tim: u128,
        inc: u128,
        nodes: u128,
        depth: u32,
        movetime: u128,
        #[cfg(feature = "datagen")] kld: f64,
    ) {
        self.use_time = tim != 0;
        self.use_nodes = nodes != 0;
        self.use_depth = depth != 0;
        self.use_move_time = movetime != 0;
        self.time = tim;
        self.increment = inc;
        self.node_lim = nodes;
        self.depth_limit = depth;
        self.move_time = movetime;
        #[cfg(feature = "datagen")]
        {
            self.min_kld = kld;
            self.use_kld = kld != 0.0;
        }
    }

    // maybe gainer idea, give slightly less for the min part
    fn time_allocated(&self, tunables: &Tunables) -> u128 {
        ((self.time as f32 / tunables.time_divisor()) as u128
            + (self.increment as f32 / tunables.inc_divisor()) as u128)
            .min(self.time)
    }

    pub fn check(
        &self,
        tim: u128,
        nodes: u128,
        depth: u32,
        tunables: &Tunables,
        #[cfg(feature = "datagen")] new_visit_distribution: &[u32],
        #[cfg(feature = "datagen")] old_visit_distribution: &[u32],
    ) -> bool {
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
        #[cfg(feature = "datagen")]
        if self.use_kld {
            // calculate kld
            let kld = calc_kld(new_visit_distribution, old_visit_distribution);

            if kld.is_some() && kld.unwrap() <= self.min_kld {
                return false;
            }
        }
        true
    }
}

#[cfg(feature = "datagen")]
fn calc_kld(new_visit_distribution: &[u32], old_visit_distribution: &[u32]) -> Option<f64> {
    let new_visits_sum = new_visit_distribution.iter().sum::<u32>();
    let old_visits_sum = old_visit_distribution.iter().sum::<u32>();

    if old_visits_sum == 0 {
        return None;
    }

    let mut kld = 0.0;

    for i in 0..new_visit_distribution.len() {
        let new_visits = new_visit_distribution[i];
        let old_visits = old_visit_distribution[i];

        if old_visits == 0 {
            return None;
        }

        let q = new_visits as f64 / new_visits_sum as f64;
        let p = old_visits as f64 / old_visits_sum as f64;

        kld += p * (p / q).ln();
    }

    Some(kld / (new_visits_sum / old_visits_sum) as f64)
}
