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
}
