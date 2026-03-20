use rand::Rng;
use parser::Problem;
use crate::model::{Solution, TimetableData};

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

        self.final_fitness_evaluation(solutions)
    }
}

impl NaiveSolver {
    pub fn new(rng: Box<dyn Rng>, population_size: usize, generations: usize, problem: Problem) -> Self {
        Self { rng, population_size, generations, data: TimetableData::new(&problem) }
    }

    fn initialize_solutions(&mut self) -> Vec<Solution> {
        let mut solutions = Vec::with_capacity(self.population_size);
        for i in 0..self.population_size {
            solutions.push(Solution::new(&self.data, &mut self.rng));
        }
        solutions
    }

    fn evaluate_solutions_fitness(&self, solutions: &[Solution]) -> Vec<f64> {
        todo!()
    }

    fn tournament_selection(&self, solutions: &[Solution], fitness: &[f64]) -> Vec<usize> {
        todo!()
    }

    fn crossover(&self, solutions: &mut [Solution], selected: Vec<usize>) {
        todo!()
    }

    fn apply_mutations(&self, solutions: &mut [Solution]) {
        todo!()
    }

    fn final_fitness_evaluation(&self, solutions: Vec<Solution>) -> Solution {
        let fitness = self.evaluate_solutions_fitness(&solutions);
        let index_of_max = fitness.iter().enumerate()
            .max_by(|(_, value0), (_, value1)| value0.partial_cmp(value1).unwrap())
            .unwrap().0;
        solutions[index_of_max].clone()
    }
}