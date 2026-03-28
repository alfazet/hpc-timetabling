use rand::{Rng, seq::index};

use crate::{fitness::Fitness, solution::Solution};

pub trait Selection {
    fn select(&mut self, solutions: &[Solution], fitness: &[Fitness]) -> Vec<usize>;
}

pub struct TournamentSelection {
    tournament_size: usize,
    rng: Box<dyn Rng>,
}

impl TournamentSelection {
    pub fn new(tournament_size: usize, rng: Box<dyn Rng>) -> Self {
        Self {
            tournament_size,
            rng,
        }
    }
}

impl Selection for TournamentSelection {
    fn select(&mut self, solutions: &[Solution], fitness: &[Fitness]) -> Vec<usize> {
        let n = solutions.len();
        let tournament_size = self.tournament_size.min(n);
        let mut selected = Vec::with_capacity(n);
        for _ in 0..n {
            let cand_idxs = index::sample(&mut self.rng, n, tournament_size);
            let best_idx = cand_idxs
                .iter()
                .min_by(|&i, &j| fitness[i].cmp(&fitness[j]))
                .expect("solutions vec shouldn't be empty");
            selected.push(best_idx);
        }

        selected
    }
}
