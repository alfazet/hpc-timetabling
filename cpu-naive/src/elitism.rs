use crate::{penalty::Penalty, solution::Solution};

pub struct Elitism {
    retain_percentage: f32,
}

impl Elitism {
    pub fn new(retain_percentage: f32) -> Self {
        Self { retain_percentage }
    }

    /// splits the vec of solutions into (the best X%, others)
    pub fn split(
        &self,
        solutions: Vec<Solution>,
        penalties: Vec<Penalty>,
    ) -> (Vec<Solution>, Vec<Penalty>, Vec<Solution>, Vec<Penalty>) {
        let mut tuples: Vec<_> = solutions.into_iter().zip(penalties).collect();
        tuples.sort_by_key(|tuple| tuple.1);
        let cut_point =
            ((tuples.len() as f32 * self.retain_percentage) as usize).min(tuples.len() - 1);
        let other = tuples.split_off(cut_point);
        let (top_solutions, top_fitness) = tuples.into_iter().unzip();
        let (other_solutions, other_fitness) = other.into_iter().unzip();

        (top_solutions, top_fitness, other_solutions, other_fitness)
    }

    pub fn elites(
        &self,
        solutions: &[Solution],
        penalties: &[Penalty],
    ) -> (Vec<Solution>, Vec<Penalty>) {
        let mut indices: Vec<usize> = (0..solutions.len()).collect();
        indices.sort_by_key(|&i| penalties[i]);

        let k = (indices.len() as f32 * self.retain_percentage) as usize;
        let mut elite_solutions = Vec::with_capacity(k);
        let mut elite_penalties = Vec::with_capacity(k);

        for &i in &indices[..k] {
            elite_solutions.push(solutions[i].clone());
            elite_penalties.push(penalties[i]);
        }

        (elite_solutions, elite_penalties)
    }

    /// replaces [Self::retain_percentage] of the worst solutions with best.
    pub fn replace_worst(
        &self,
        elites: &[Solution],
        offspring: &mut Vec<Solution>,
        elite_penalties: &[Penalty],
        offspring_penalties: &mut Vec<Penalty>,
    ) {
        let mut indices: Vec<usize> = (0..offspring.len()).collect();

        indices.sort_by_key(|&i| offspring_penalties[i]);

        let worst = &indices[indices.len() - elites.len()..];

        for (idx, (elite, &pen)) in worst.iter().zip(elites.iter().zip(elite_penalties)) {
            offspring[*idx] = elite.clone();
            offspring_penalties[*idx] = pen;
        }
    }
}
