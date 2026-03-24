use crate::{model::TimetableData, solution::Solution};
use rand::Rng;

pub trait Solver {
    fn solve(&mut self) -> Solution;
}

pub struct NaiveSolver {
    rng: Box<dyn Rng>,
    population_size: usize,
    generations: usize,
    data: TimetableData,
}

impl Solver for NaiveSolver {
    fn solve(&mut self) -> Solution {
        let mut solutions = self.initialize_solutions();
        for generation in 0..self.generations {
            let fitness = self.evaluate_solutions_fitness(&solutions);
            let selected = self.tournament_selection(&solutions, &fitness);
            self.crossover(&mut solutions, selected);
            self.apply_mutations(&mut solutions);
        }
        let final_fitness = self.evaluate_solutions_fitness(&solutions);
        let max_idx = final_fitness
            .into_iter()
            .enumerate()
            .max_by(|(_, f1), (_, f2)| f1.partial_cmp(f2).unwrap())
            .expect("solutions vec shouldn't be empty")
            .0;

        solutions[max_idx].clone()
    }
}

impl NaiveSolver {
    pub fn new(
        rng: Box<dyn Rng>,
        population_size: usize,
        generations: usize,
        data: TimetableData,
    ) -> Self {
        Self {
            rng,
            population_size,
            generations,
            data,
        }
    }

    fn initialize_solutions(&mut self) -> Vec<Solution> {
        let mut solutions = Vec::with_capacity(self.population_size);
        for _ in 0..self.population_size {
            solutions.push(Solution::new(&self.data, &mut self.rng));
        }

        solutions
    }

    fn student_assignment_penalty(&self, sol: &Solution) -> f64 {
        0.0
    }

    fn solution_fitness(&self, sol: &Solution) -> f64 {
        let mut penalty = 0.0;
        penalty += self.student_assignment_penalty(sol);

        // returned fitness will be some function of penalty
        // (small penalty -> huge fitness, more than small penalty -> low fitness)

        penalty
    }

    fn evaluate_solutions_fitness(&self, solutions: &[Solution]) -> Vec<f64> {
        // parallelizing this will be a change from `iter` to `par_iter`
        solutions
            .iter()
            .map(|sol| self.solution_fitness(sol))
            .collect()
    }

    fn tournament_selection(&self, solutions: &[Solution], fitness: &[f64]) -> Vec<usize> {
        // TODO
        vec![0; solutions.len()]
    }

    fn crossover(&self, solutions: &mut [Solution], selected: Vec<usize>) {
        // TODO
    }

    fn apply_mutations(&self, solutions: &mut [Solution]) {
        // TODO
    }
}
