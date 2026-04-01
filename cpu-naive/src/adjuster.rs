use crate::{penalty::Penalty, stats::GenerationStats};

pub struct Adjuster {
    max_no_improvement: usize,
}

impl Adjuster {
    pub fn new(max_no_improvement: usize) -> Self {
        Self { max_no_improvement }
    }

    pub fn adjust(&self, stats: &GenerationStats, mutation: &mut f32, crossover: &mut f32) {
        let n = stats.no_improvement as f32;
        let m = self.max_no_improvement as f32;
        let r = 1.0 + (n - m) / m * 0.001;
        dbg!(r);

        macro_rules! update_print {
            ($v:ident, $max:expr) => {
                eprint!("{}: {:.4} ->", stringify!($v), $v);
                *$v = (*$v * r).min($max);
                eprintln!(" {:.4}", $v);
            };
        }
        update_print!(mutation, 0.2);
        update_print!(crossover, 1.0);
    }
}
