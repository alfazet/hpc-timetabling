use crate::{
    model::{RoomData, TimetableData},
    solution::Solution,
};
use parser::timeslots::TimeSlots;
use rand::{
    Rng, RngExt,
    seq::{SliceRandom, index},
};

pub trait Solver {
    fn solve(&mut self) -> EvaluatedSolution;
}

pub struct EvaluatedSolution {
    pub inner: Solution,
    pub fitness: f64,
}

pub struct NaiveSolver {
    rng: Box<dyn Rng>,
    population_size: usize,
    generations: usize,
    data: TimetableData,
}

impl Solver for NaiveSolver {
    fn solve(&mut self) -> EvaluatedSolution {
        let tournament_size = 5; // TODO: parametrize the function

        let mut solutions = self.initialize_solutions();
        for generation in 0..self.generations {
            let fitness = self.evaluate_solutions_fitness(&solutions);
            let selected = self.tournament_selection(&solutions, &fitness, tournament_size);
            self.crossover(&mut solutions, selected);
            self.apply_mutations(&mut solutions);
        }
        let final_fitness = self.evaluate_solutions_fitness(&solutions);
        let max_idx = final_fitness
            .iter()
            .enumerate()
            .max_by(|(_, f1), (_, f2)| f1.partial_cmp(f2).unwrap())
            .expect("solutions vec shouldn't be empty")
            .0;

        EvaluatedSolution {
            inner: solutions[max_idx].clone(),
            fitness: final_fitness[max_idx],
        }
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

    fn timeslots_overlap(a: &TimeSlots, b: &TimeSlots) -> bool {
        let shared_weeks = a.weeks.0 & b.weeks.0;
        let shared_days = a.days.0 & b.days.0;
        if shared_weeks == 0 || shared_days == 0 {
            return false;
        }

        a.start < b.start + b.length && b.start < a.start + a.length
    }

    fn travel_time_between(rooms: &[RoomData], room_a: usize, room_b: usize) -> u32 {
        if room_a == room_b {
            return 0;
        }

        rooms[room_a]
            .travels
            .iter()
            .find(|t| t.dest_room_idx == room_b)
            .map(|t| t.travel_time)
            .unwrap_or(0)
    }

    fn insufficient_travel_time(a: &TimeSlots, b: &TimeSlots, travel: u32) -> bool {
        let shared_weeks = a.weeks.0 & b.weeks.0;
        let shared_days = a.days.0 & b.days.0;
        if shared_weeks == 0 || shared_days == 0 {
            return false;
        }
        let a_end = a.start + a.length;
        let b_end = b.start + b.length;
        let gap = if a_end <= b.start {
            b.start - a_end
        } else if b_end <= a.start {
            a.start - b_end
        } else {
            return false;
        };

        gap < travel
    }

    fn student_assignment_penalty(&self, sol: &Solution) -> u64 {
        let mut n_conflicts: u64 = 0;
        let mut classes_per_student = vec![Vec::new(); self.data.students.len()];
        for (class_idx, student_list) in sol.students_in_classes.iter().enumerate() {
            for &student_idx in student_list {
                classes_per_student[student_idx].push(class_idx);
            }
        }
        for student_classes in &classes_per_student {
            for i in 0..student_classes.len() {
                for j in (i + 1)..student_classes.len() {
                    let ci = student_classes[i];
                    let cj = student_classes[j];
                    let time_a = &sol.times[ci].times;
                    let time_b = &sol.times[cj].times;
                    if Self::timeslots_overlap(time_a, time_b) {
                        n_conflicts += 1;
                    } else {
                        let travel = match (&sol.rooms[ci], &sol.rooms[cj]) {
                            (Some(room_a), Some(room_b)) => Self::travel_time_between(
                                &self.data.rooms,
                                room_a.room_idx,
                                room_b.room_idx,
                            ),
                            _ => 0,
                        };
                        if travel > 0 && Self::insufficient_travel_time(time_a, time_b, travel) {
                            n_conflicts += 1;
                        }
                    }
                }
            }
        }

        n_conflicts * self.data.optimization.student as u64
    }

    /// add up all the penalties for classes:
    /// - having more students than allowed by their limit
    /// - taking place in rooms that don't have enough capacity
    /// - taking place in rooms that are unavailable in chosen timeslots
    /// - time intervals of two classes overlap in the same room
    /// - distribution constraints: SameStart, SameTime, ...
    fn class_assignment_penalty(&self, sol: &Solution) -> u64 {
        0
    }

    fn solution_fitness(&self, sol: &Solution) -> f64 {
        let mut penalty = 0;
        penalty += self.student_assignment_penalty(sol);
        penalty += self.class_assignment_penalty(sol);

        // TODO: this should probably be some fancier function
        1.0 / (penalty as f64 + 1.0)
    }

    fn evaluate_solutions_fitness(&self, solutions: &[Solution]) -> Vec<f64> {
        // parallelizing this should be a change from `iter` to `par_iter`
        solutions
            .iter()
            .map(|sol| self.solution_fitness(sol))
            .collect()
    }

    fn tournament_selection(
        &mut self,
        solutions: &[Solution],
        fitness: &[f64],
        tournament_size: usize,
    ) -> Vec<usize> {
        let n = solutions.len();
        let tournament_size = tournament_size.min(n);
        let mut selected = Vec::with_capacity(n);
        for _ in 0..n {
            let cand_idxs = index::sample(&mut self.rng, n, tournament_size);
            let best_idx = cand_idxs
                .iter()
                .max_by(|&i, &j| fitness[i].partial_cmp(&fitness[j]).unwrap())
                .unwrap_or(0);
            selected.push(best_idx);
        }

        selected
    }

    fn crossover(&mut self, solutions: &mut Vec<Solution>, selected: Vec<usize>) {
        let n_classes = self.data.classes.len();
        let n_solutions = solutions.len();
        let mut new_solutions = Vec::with_capacity(n_solutions);
        for pair in selected.chunks(2) {
            let parent_a = &solutions[pair[0]];
            let parent_b = if pair.len() == 2 {
                &solutions[pair[1]]
            } else {
                // edge case: odd population size
                new_solutions.push(solutions[pair[0]].clone());
                break;
            };

            // the child takes a random proportion of values from both parents
            let cut_point = self.rng.random_range(1..n_classes);
            let child1 = Solution {
                times: [&parent_a.times[..cut_point], &parent_b.times[cut_point..]].concat(),
                rooms: [&parent_a.rooms[..cut_point], &parent_b.rooms[cut_point..]].concat(),
                students_in_classes: [
                    &parent_a.students_in_classes[..cut_point],
                    &parent_b.students_in_classes[cut_point..],
                ]
                .concat(),
            };
            let child2 = Solution {
                times: [&parent_b.times[..cut_point], &parent_a.times[cut_point..]].concat(),
                rooms: [&parent_b.rooms[..cut_point], &parent_a.rooms[cut_point..]].concat(),
                students_in_classes: [
                    &parent_b.students_in_classes[..cut_point],
                    &parent_a.students_in_classes[cut_point..],
                ]
                .concat(),
            };
            new_solutions.push(child1);
            new_solutions.push(child2);
        }
        new_solutions.shuffle(&mut self.rng);
        new_solutions.truncate(n_solutions);

        *solutions = new_solutions;
    }

    fn apply_mutations(&self, solutions: &mut [Solution]) {
        // TODO
    }
}
