#[derive(Clone, Default, Debug)]
struct Tunable {
    val: f32,
    min: f32,
    max: f32,
}

impl Tunable {
    fn set(&mut self, val: i32) {
        let actual = val as f32 / 1000.0;
        self.val = actual.clamp(self.min, self.max);
    }

    fn print_option(&self, name: &str) {
        println!(
            "option name {} type spin default {:.0} min {:.0} max {:.0}",
            name,
            self.val * 1000.0,
            self.min * 1000.0,
            self.max * 1000.0,
        );
    }

    fn list(&self, name: &str, step: f32, r: f32) {
        println!(
            "{}, {}, {}, {}, {}, {}",
            name,
            (self.val * 1000.0) as i32,
            (self.min * 1000.0) as i32,
            (self.max * 1000.0) as i32,
            (step * 1000.0) as i32,
            r,
        );
    }
}

macro_rules! make_tunables {
    ($($name:ident = $val:expr, $min:expr, $max:expr, $step:expr, $r:expr;)*) => {
        #[derive(Clone, Debug)]
        pub struct Tunables {
            $($name: Tunable,)*
        }

        impl Tunables {
            $(
                pub const fn $name(&self) -> f32 {
                    self.$name.val
                }
            )*

            pub fn print_options(&self) {
                $(self.$name.print_option(stringify!($name));)*
            }

            pub fn set(&mut self, name: &str, val: i32) -> Result<(), String> {
                match name {
                    $(stringify!($name) => {
                        self.$name.set(val);
                        Ok(())
                    },)*
                    _ => Err(format!("Unknown tunable option: {}", name)),
                }
            }

            pub fn list(&self) {
                $(self.$name.list(stringify!($name), $step, $r);)*
            }
        }

        impl Tunables {
            pub fn new() -> Self {
                Self {
                    $($name: Tunable {
                        val: $val,
                        min: $min,
                        max: $max,
                    },)*
                }
            }
        }
    };
}

make_tunables! {
    default_cpuct = 0.5, 0.0, 10.0, 0.05, 0.002;
    root_cpuct = 0.7, 0.0, 10.0, 0.05, 0.002;
    gini_base = 0.463, 0.0, 2.0, 0.05, 0.002;
    gini_log_mult = 1.567, 0.0, 3.0, 0.16, 0.002;
    gini_min = 2.26, 0.0, 4.0, 0.20, 0.002;
    default_pst = 1.0, 0.1, 2.0, 0.06, 0.002;
    root_pst_bonus = 2.5, 0.1, 5.0, 0.25, 0.002;
    time_divisor = 20.0, 1.0, 50.0, 2.25, 0.002;
    inc_divisor = 2.0, 1.0, 5.0, 0.225, 0.002;
    cpuct_visits_scale = 40.0, 1.0, 512.0, 3.2, 0.002;
}
