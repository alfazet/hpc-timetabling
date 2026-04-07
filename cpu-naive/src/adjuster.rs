use crate::penalty::Penalty;

pub struct GenerationStats {
    generation: usize,
    min_penalty: Option<Penalty>,
    no_improvement: usize,
}

impl GenerationStats {
    pub fn new() -> Self {
        Self {
            generation: 0,
            min_penalty: None,
            no_improvement: 0,
        }
    }

    pub fn update(&mut self, current_min_penalty: Penalty) {
        if let Some(p) = self.min_penalty
            && current_min_penalty == p
        {
            self.no_improvement += 1;
        } else {
            self.no_improvement = 0;
        }
        self.generation += 1;
        self.min_penalty = Some(current_min_penalty);
    }

    pub fn print_logs(&self) {
        let Some(p) = self.min_penalty else {
            return;
        };
        eprintln!("min penalty after {} generations: {}", self.generation, p);
    }
}

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
