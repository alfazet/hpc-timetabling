use rand::{Rng, RngExt, seq::SliceRandom};

use crate::solution::Solution;

pub trait Crossover {
    fn crossover(&mut self, rng: &mut dyn Rng, solutions: &mut Vec<Solution>, selected: &[usize]);
    fn probability(&mut self) -> &mut f64;
}

pub struct OnePointCrossover {
    probability: f64,
}

impl OnePointCrossover {
    pub fn new(probability: f64) -> Self {
        Self { probability }
    }
}

pub struct UniformCrossover {
    probability: f64,
}

impl UniformCrossover {
    pub fn new(probability: f64) -> Self {
        Self { probability }
    }
}

impl Crossover for UniformCrossover {
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

            let mut child1 = parent_b.clone();
            let mut child2 = parent_a.clone();

            for class_idx in 0..n_classes {
                let a_bias = 0.5;
                if rng.random_bool(a_bias) {
                    child1.times[class_idx] = parent_b.times[class_idx].clone();
                    child1.rooms[class_idx] = parent_b.rooms[class_idx].clone();
                    child2.times[class_idx] = parent_a.times[class_idx].clone();
                    child2.rooms[class_idx] = parent_a.rooms[class_idx].clone();
                }
            }

            new_solutions.push(child1);
            new_solutions.push(child2);
        }

        new_solutions.shuffle(rng);
        new_solutions.truncate(n_solutions);

        *solutions = new_solutions;
    }

    fn probability(&mut self) -> &mut f64 {
        &mut self.probability
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

            if !rng.random_bool(self.probability) {
                new_solutions.push(parent_a.clone());
                new_solutions.push(parent_b.clone());
                continue;
            }

            let times_rooms_cut = rng.random_range(1..n_classes);
            let n_students = parent_a.config_preferences.len();
            let student_cut = if n_students > 1 {
                rng.random_range(1..n_students)
            } else {
                n_students
            };

            let child1 = Solution {
                times: [
                    &parent_a.times[..times_rooms_cut],
                    &parent_b.times[times_rooms_cut..],
                ]
                .concat(),
                rooms: [
                    &parent_a.rooms[..times_rooms_cut],
                    &parent_b.rooms[times_rooms_cut..],
                ]
                .concat(),
                config_preferences: [
                    &parent_a.config_preferences[..student_cut],
                    &parent_b.config_preferences[student_cut..],
                ]
                .concat(),
            };
            let child2 = Solution {
                times: [
                    &parent_b.times[..times_rooms_cut],
                    &parent_a.times[times_rooms_cut..],
                ]
                .concat(),
                rooms: [
                    &parent_b.rooms[..times_rooms_cut],
                    &parent_a.rooms[times_rooms_cut..],
                ]
                .concat(),
                config_preferences: [
                    &parent_b.config_preferences[..student_cut],
                    &parent_a.config_preferences[student_cut..],
                ]
                .concat(),
            };
            new_solutions.push(child1);
            new_solutions.push(child2);
        }
        new_solutions.shuffle(rng);
        new_solutions.truncate(n_solutions);

        *solutions = new_solutions;
    }

    fn probability(&mut self) -> &mut f64 {
        &mut self.probability
    }
}
