use std::fs::File;
use std::io::Write;

use crate::penalty::Penalty;

pub struct GenerationStats {
    pub log_file: Option<File>,
    pub generation: usize,
    pub min_penalty: Option<Penalty>,
    pub mean_penalty_hard: Option<f32>,
    pub no_improvement: usize,
}

impl GenerationStats {
    pub fn new() -> Self {
        Self {
            log_file: File::create("visualization/metrics.csv").ok(),
            generation: 0,
            min_penalty: None,
            mean_penalty_hard: None,
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
        if let Some(prev_mean) = self.mean_penalty_hard {
            self.mean_penalty_hard = Some(
                (prev_mean * (self.generation as f32 - 1.0)
                    + self.min_penalty.unwrap().hard as f32)
                    / self.generation as f32,
            );
        } else {
            self.mean_penalty_hard = Some(self.min_penalty.unwrap().hard as f32);
        }
    }

    pub fn print_logs(&mut self) {
        let (Some(p), Some(pp)) = (self.min_penalty, self.mean_penalty_hard) else {
            return;
        };
        eprintln!("min penalty after {} generations: {}", self.generation, p);
        if let Some(file) = &mut self.log_file {
            writeln!(file, "{},{},{},{}", self.generation, p.hard, p.soft, pp).unwrap();
        }
    }
}
