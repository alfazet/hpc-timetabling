use crate::penalty::Penalty;

pub struct GenerationStats {
    generation: usize,
    stagnation: usize,
    progress: usize,
    min_penalty: Option<Penalty>,
}

impl GenerationStats {
    pub fn new() -> Self {
        Self {
            generation: 0,
            stagnation: 0,
            progress: 0,
            min_penalty: None,
        }
    }

    pub fn update(&mut self, current_min_penalty: Penalty) {
        if let Some(p) = self.min_penalty
            && current_min_penalty == p
        {
            self.stagnation += 1;
            self.progress = 0;
        } else {
            self.progress += 1;
            self.stagnation = 0;
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
    /// how much to increase mutation rate / decrease crossover rate per stagnating generation
    delta: f64,
    min_mutation: f64,
    max_mutation: f64,
    min_crossover: f64,
    max_crossover: f64,
}

impl Adjuster {
    pub fn new(
        delta: f64,
        min_mutation: f64,
        max_mutation: f64,
        min_crossover: f64,
        max_crossover: f64,
    ) -> Self {
        Self {
            delta,
            min_mutation,
            max_mutation,
            min_crossover,
            max_crossover,
        }
    }

    pub fn adjust(&self, stats: &GenerationStats, mutation: &mut f64, crossover: &mut f64) {
        match stats.stagnation {
            0 => self.more_focus(stats.progress, mutation, crossover),
            n => self.more_exploration(n, mutation, crossover),
        }
    }

    fn more_focus(&self, n: usize, mutation: &mut f64, crossover: &mut f64) {
        let change = self.delta
            * match n {
                ..2 => 1.0,
                2..5 => 1.25,
                5..10 => 1.5,
                _ => 1.75,
            };
        *mutation = (*mutation - change).clamp(self.min_mutation, self.max_mutation);
        *crossover = (*crossover + change).clamp(self.min_crossover, self.max_crossover);
        eprintln!(
            "progressing for {} generations, -{:.4} to mutation rate ({:.4}), +{:.4} to crossover rate ({:.4})",
            n, change, mutation, change, crossover
        );
    }

    fn more_exploration(&self, n: usize, mutation: &mut f64, crossover: &mut f64) {
        let change = self.delta
            * match n {
                ..2 => 1.0,
                2..5 => 1.25,
                5..10 => 1.5,
                _ => 1.75,
            };
        *mutation = (*mutation + change).clamp(self.min_mutation, self.max_mutation);
        *crossover = (*crossover - change).clamp(self.min_crossover, self.max_crossover);
        eprintln!(
            "stagnating for {} generations, +{:.4} to mutation rate ({:.4}), -{:.4} to crossover rate ({:.4})",
            n, change, mutation, change, crossover
        );
    }
}
