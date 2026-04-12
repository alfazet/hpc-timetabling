use crate::adjuster::{Adjuster, GenerationStats};
use crate::assigner::{self, StudentAssignment};
use crate::distribution::Distribution;
use crate::local_search::HillClimbing;
use crate::{
    crossover::Crossover,
    elitism::Elitism,
    model::{RoomData, TimetableData},
    mutation::Mutation,
    penalty::Penalty,
    selection::Selection,
    solution::Solution,
};
use crate::{evaluator, utils};
use parser::timeslots::TimeSlots;
use rand::Rng;
use rayon::prelude::*;

pub trait Solver {
    fn solve(&mut self, rng: &mut dyn Rng) -> EvaluatedSolution;
}

pub struct EvaluatedSolution {
    pub inner: Solution,
    pub penalty: Penalty,
    pub student_assignment: StudentAssignment,
}

pub struct NaiveSolver<S, C, M>
where
    S: Selection,
    C: Crossover,
    M: Mutation,
{
    population_size: usize,
    generations: usize,
    data: TimetableData,
    elitism: Elitism,
    selection: S,
    crossover: C,
    mutation: M,
    stats: GenerationStats,
    adjuster: Adjuster,
    hill_climbing: HillClimbing,
    last_penalties: Option<Vec<Penalty>>,
}

impl<S, C, M> Solver for NaiveSolver<S, C, M>
where
    S: Selection + Sync,
    C: Crossover + Sync,
    M: Mutation + Sync,
{
    fn solve(&mut self, rng: &mut dyn Rng) -> EvaluatedSolution {
        let mut solutions = self.initialize_solutions(rng);
        for _ in 0..self.generations {
            let assignments = self.find_assignments(&solutions);

            // take the penalties from the end of last generation, otherwise we
            // would be calculating them twice per generation
            let mut penalties = if let Some(p) = self.last_penalties.clone() {
                p
            } else {
                let p = self.evaluate_solutions_penalties(&solutions, &assignments);
                self.last_penalties = Some(p.clone());
                p
            };
            self.hill_climbing
                .optimize(&mut solutions, &mut penalties, &self.data);
            let (elites, elite_penalties) = self.elitism.elites(&solutions, &penalties);

            let selected = self.selection.select(rng, &solutions, &penalties);
            self.crossover.crossover(rng, &mut solutions, &selected);
            self.mutation.mutate(rng, &mut solutions, &self.data);

            // TODO: cache last assignments like last fitness, also requires
            // mutating them in replace_worst
            let assignments = self.find_assignments(&solutions);
            let mut penalties = self.evaluate_solutions_penalties(&solutions, &assignments);
            self.elitism
                .replace_worst(&elites, &elite_penalties, &mut solutions, &mut penalties);

            let min_penalty = penalties
                .iter()
                .min()
                .expect("solutions vec shouldn't be empty");

            self.stats.update(*min_penalty);
            self.stats.print_logs();
            self.adjuster.adjust(
                &self.stats,
                self.mutation.probability(),
                self.crossover.probability(),
            );

            self.last_penalties = Some(penalties);
        }

        let final_assignments = self.find_assignments(&solutions);
        let final_penalties = self.evaluate_solutions_penalties(&solutions, &final_assignments);
        let min_idx = final_penalties
            .iter()
            .enumerate()
            .min_by(|(_, f1), (_, f2)| f1.cmp(f2))
            .expect("solutions vec shouldn't be empty")
            .0;

        let (best_solution, best_assignment, min_penalty) = {
            let min_penalty = final_penalties[min_idx];
            let mut assignments = final_assignments;

            (
                solutions.swap_remove(min_idx),
                assignments.swap_remove(min_idx),
                min_penalty,
            )
        };

        EvaluatedSolution {
            inner: best_solution,
            penalty: min_penalty,
            student_assignment: best_assignment,
        }
    }
}

impl<S, C, M> NaiveSolver<S, C, M>
where
    S: Selection + Sync,
    C: Crossover + Sync,
    M: Mutation + Sync,
{
    pub fn new(
        population_size: usize,
        generations: usize,
        data: TimetableData,
        elitism: Elitism,
        selection: S,
        crossover: C,
        mutation: M,
        hill_climbing: HillClimbing,
    ) -> Self {
        Self {
            population_size,
            generations,
            data,
            elitism,
            selection,
            crossover,
            mutation,
            stats: GenerationStats::new(),
            adjuster: Adjuster::new((generations / 50).max(1)),
            last_penalties: None,
            hill_climbing,
        }
    }

    fn initialize_solutions(&mut self, rng: &mut dyn Rng) -> Vec<Solution> {
        let mut solutions = Vec::with_capacity(self.population_size);
        for _ in 0..self.population_size {
            solutions.push(Solution::new(&self.data, rng));
        }

        solutions
    }

    fn find_assignments(&self, solutions: &[Solution]) -> Vec<StudentAssignment> {
        solutions
            .par_iter()
            .map(|sol| assigner::assign_students(&self.data, sol))
            .collect()
    }

    fn evaluate_solutions_penalties(
        &self,
        solutions: &[Solution],
        assignments: &[StudentAssignment],
    ) -> Vec<Penalty> {
        solutions
            .par_iter()
            .zip(assignments)
            .map(|(sol, assignment)| evaluator::evaluate(sol, &self.data, assignment))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use parser::Problem;

    use crate::{
        assigner::StudentAssignment, crossover::OnePointCrossover, elitism::Elitism,
        local_search::HillClimbing, model::TimetableData, mutation::BasicMutation,
        selection::TournamentSelection, solver::NaiveSolver,
    };

    fn solver() -> NaiveSolver<TournamentSelection, OnePointCrossover, BasicMutation> {
        let xml = include_str!("../../data/test-data/students-test.xml");
        let problem = Problem::parse(xml).unwrap();
        let data = TimetableData::new(problem);
        let solver = NaiveSolver::new(
            1,
            1,
            data,
            Elitism::new(0.0),
            TournamentSelection::new(1),
            OnePointCrossover::new(0.5),
            BasicMutation::new(0.0),
            HillClimbing::new(0, 0.0),
        );
        solver
    }

    // TODO: refactor tests (move them to the evaluator)
    /*
    #[test]
    fn test_students_not_enrolled_in_parent_penalty_empty() {
        let solver = solver();
        let assignment = StudentAssignment {
            students_in_classes: vec![vec![]; 3],
        };

        let penalty = students_not_enrolled_in_parent_penalty(&assignment);

        assert_eq!(penalty, 0);
    }

    #[test]
    fn test_students_not_enrolled_in_parent_penalty() {
        let solver = solver();
        let assignment = StudentAssignment {
            students_in_classes: vec![vec![], vec![], vec![0, 1]],
        };

        let penalty = solver.students_not_enrolled_in_parent_penalty(&assignment);

        assert_eq!(penalty, 2);
    }

    #[test]
    fn test_students_not_enrolled_in_parent_penalty_correct() {
        let solver = solver();
        let assignment = StudentAssignment {
            students_in_classes: vec![vec![0, 1], vec![1], vec![0, 1]],
        };

        let penalty = solver.students_not_enrolled_in_parent_penalty(&assignment);

        assert_eq!(penalty, 0);
    }

    #[test]
    fn students_not_enrolled_in_exactly_one_per_subpart_empty() {
        let solver = solver();
        let assignment = StudentAssignment {
            students_in_classes: vec![vec![]; 3],
        };

        let penalty = solver.students_not_enrolled_in_exactly_one_per_subpart(&assignment);

        assert_eq!(penalty, 4);
    }

    #[test]
    fn students_not_enrolled_in_exactly_one_per_subpart_too_much() {
        let solver = solver();
        let assignment = StudentAssignment {
            students_in_classes: vec![vec![0, 1], vec![0], vec![0, 1]],
        };

        let penalty = solver.students_not_enrolled_in_exactly_one_per_subpart(&assignment);

        assert_eq!(penalty, 1);
    }

    #[test]
    fn students_not_enrolled_in_exactly_one_per_subpart_missing() {
        let solver = solver();
        let assignment = StudentAssignment {
            students_in_classes: vec![vec![0, 1], vec![], vec![1]],
        };

        let penalty = solver.students_not_enrolled_in_exactly_one_per_subpart(&assignment);

        assert_eq!(penalty, 1);
    }

    #[test]
    fn students_not_enrolled_in_exactly_one_per_subpart_correct() {
        let solver = solver();
        let assignment = StudentAssignment {
            students_in_classes: vec![vec![0], vec![1], vec![0, 1]],
        };

        let penalty = solver.students_not_enrolled_in_exactly_one_per_subpart(&assignment);

        assert_eq!(penalty, 0);
    }
    */
}
