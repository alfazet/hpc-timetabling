use rand::{Rng, RngExt, seq::SliceRandom};

use crate::solution::Solution;

pub trait Crossover {
    fn crossover(&mut self, rng: &mut dyn Rng, solutions: &mut Vec<Solution>, selected: &[usize]);
    fn probability(&mut self) -> &mut f32;
}

pub struct OnePointCrossover {
    probability: f32,
}

impl OnePointCrossover {
    pub fn new(probability: f32) -> Self {
        Self { probability }
    }
}

impl Crossover for OnePointCrossover {
    fn crossover(&mut self, rng: &mut dyn Rng, solutions: &mut Vec<Solution>, selected: &[usize]) {
        debug_assert!(!solutions.is_empty());
        debug_assert!(selected.len() <= solutions.len());

        let n_classes = solutions[0].times.len();
        let n_solutions = solutions.len();
        let mut new_solutions = Vec::with_capacity(n_solutions);

        let (chunks, rem) = selected.as_chunks();
        if !rem.is_empty() {
            new_solutions.push(solutions[rem[0]].clone());
        }
        for [a, b] in chunks {
            let parent_a = &solutions[*a];
            let parent_b = &solutions[*b];

            if rng.random_range(0.0..1.0) > self.probability {
                new_solutions.push(parent_a.clone());
                new_solutions.push(parent_b.clone());
                continue;
            }

            // the child takes a random proportion of values from both parents
            let cut_point = rng.random_range(1..n_classes);
            let child1 = Solution {
                times: [&parent_a.times[..cut_point], &parent_b.times[cut_point..]].concat(),
                rooms: [&parent_a.rooms[..cut_point], &parent_b.rooms[cut_point..]].concat(),
            };
            let child2 = Solution {
                times: [&parent_b.times[..cut_point], &parent_a.times[cut_point..]].concat(),
                rooms: [&parent_b.rooms[..cut_point], &parent_a.rooms[cut_point..]].concat(),
            };
            new_solutions.push(child1);
            new_solutions.push(child2);
        }
        new_solutions.shuffle(rng);
        new_solutions.truncate(n_solutions);

        *solutions = new_solutions;
    }

    fn probability(&mut self) -> &mut f32 {
        &mut self.probability
    }
}
