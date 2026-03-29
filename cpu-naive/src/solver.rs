use crate::{
    crossover::Crossover,
    elitism::Elitism,
    fitness::Fitness,
    model::{RoomData, TimetableData},
    mutation::Mutation,
    selection::Selection,
    solution::Solution,
};
use parser::timeslots::TimeSlots;
use rand::Rng;
use crate::distribution::Distribution;

pub trait Solver {
    fn solve(&mut self) -> EvaluatedSolution;
}

pub struct EvaluatedSolution {
    pub inner: Solution,
    pub fitness: Fitness,
}

pub struct NaiveSolver<S, C, M>
where
    S: Selection,
    C: Crossover,
    M: Mutation,
{
    rng: Box<dyn Rng>,
    population_size: usize,
    generations: usize,
    data: TimetableData,
    elitism: Elitism,
    selection: S,
    crossover: C,
    mutation: M,
}

impl<S, C, M> Solver for NaiveSolver<S, C, M>
where
    S: Selection,
    C: Crossover,
    M: Mutation,
{
    fn solve(&mut self) -> EvaluatedSolution {
        let mut solutions = self.initialize_solutions();
        for generation in 0..self.generations {
            let mut fitness = self.evaluate_solutions_fitness(&solutions);
            let (top_solutions, top_fitness, mut other_solutions, mut other_fitness) =
                self.elitism.split(solutions, fitness);
            let selected = self.selection.select(&other_solutions, &other_fitness);
            self.crossover.crossover(&mut other_solutions, &selected);
            self.mutation.mutate(&mut other_solutions, &self.data);

            // merge unchanged top solutions with crossed-over/mutated others
            other_solutions.extend(top_solutions);
            solutions = other_solutions;
            other_fitness.extend(top_fitness);
            fitness = other_fitness;

            let min_fitness = fitness
                .iter()
                .min()
                .expect("solutions vec shouldn't be empty");
            eprintln!(
                "min penalty after {} generations: {}",
                generation, min_fitness
            );
        }
        let final_fitness = self.evaluate_solutions_fitness(&solutions);
        let min_idx = final_fitness
            .iter()
            .enumerate()
            .min_by(|(_, f1), (_, f2)| f1.cmp(f2))
            .expect("solutions vec shouldn't be empty")
            .0;

        EvaluatedSolution {
            inner: solutions[min_idx].clone(),
            fitness: final_fitness[min_idx],
        }
    }
}

impl<S, C, M> NaiveSolver<S, C, M>
where
    S: Selection,
    C: Crossover,
    M: Mutation,
{
    pub fn new(
        rng: Box<dyn Rng>,
        population_size: usize,
        generations: usize,
        data: TimetableData,
        elitism: Elitism,
        selection: S,
        crossover: C,
        mutation: M,
    ) -> Self {
        Self {
            rng,
            population_size,
            generations,
            data,
            elitism,
            selection,
            crossover,
            mutation,
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

    fn student_assignment_conflicts(&self, sol: &Solution) -> u32 {
        let mut n_conflicts = 0;
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

        n_conflicts
    }

    /// - TODO: sf else?
    fn classes_hard_penalties(&self, sol: &Solution) -> u32 {
        let mut n_violations = 0;

        n_violations += self.classes_student_limits_penalty(&sol);
        n_violations += self.rooms_capacity_limits_penalty(&sol);
        n_violations += self.classes_in_unavailable_rooms_penalty(&sol);
        n_violations += self.time_intervals_overlap_penalty(&sol);

        n_violations
    }

    /// counts the hard violations for classes having more students
    /// than allowed by their limit
    fn classes_student_limits_penalty(&self, sol: &Solution) -> u32 {
        sol.students_in_classes.iter().enumerate().map(|(index, class)| {
            if let Some(limit) = self.data.classes[index].limit {
                if class.len() > limit as usize {
                    return 1;
                }
            }
            0
        }).sum()
    }

    /// counts the hard violations for classes taking place
    /// in rooms that don't have enough capacity
    fn rooms_capacity_limits_penalty(&self, sol: &Solution) -> u32 {
        sol.students_in_classes.iter().enumerate().map(|(index, class)| {
            if let Some(room_option) = &sol.rooms[index] {
                if self.data.rooms[room_option.room_idx].capacity < class.len() as u32 {
                    return 1;
                }
            }
            0
        }).sum()
    }

    /// counts the hard violations for classes taking place
    /// in rooms that are unavailable in chosen timeslots
    fn classes_in_unavailable_rooms_penalty(&self, sol: &Solution) -> u32 {
        sol.times.iter().enumerate().map(|(index, time_option)| {
            if let Some(room_option) = &sol.rooms[index] {
                let unavailabilities = &self.data.rooms[room_option.room_idx].unavailabilities;
                let times = &time_option.times;
                if unavailabilities.iter().any(|unavailability| {
                    Self::timeslots_overlap(unavailability, times)
                }) {
                    return 1;
                }
            }
            0
        }).sum()
    }

    /// counts the hard violations -- time intervals of two
    /// classes overlap in the same room
    fn time_intervals_overlap_penalty(&self, sol: &Solution) -> u32 {
        sol.rooms.iter().enumerate().map(|(index, room_option)| {
            if let Some(room_idx) = room_option.as_ref().map(|r| r.room_idx) {
                for i in index + 1..sol.rooms.len() {
                    if let Some(i_room_idx) = sol.rooms[i].as_ref().map(|r| r.room_idx) {
                        if room_idx == i_room_idx {
                            if Self::timeslots_overlap(&sol.times[index].times, &sol.times[i].times) {
                                return 1;
                            }
                        }
                    }
                }
            }
            0
        }).sum()
    }

    fn rooms_penalty(&self, sol: &Solution) -> u32 {
        sol.rooms.iter().flatten().map(|r| r.penalty).sum()
    }

    fn times_penalty(&self, sol: &Solution) -> u32 {
        sol.times.iter().map(|t| t.penalty).sum()
    }

    fn solution_fitness(&self, sol: &Solution) -> Fitness {
        let mut fitness = Fitness::new();

        let clas = self.classes_hard_penalties(sol);
        fitness.hard += clas;

        let stud = self.student_assignment_conflicts(sol);
        fitness.soft += stud * self.data.optimization.student;

        let room = self.rooms_penalty(sol);
        fitness.soft += room * self.data.optimization.room;

        let time = self.times_penalty(sol);
        fitness.soft += time * self.data.optimization.time;

        let dist = Distribution::new(&self.data, sol).calculate_penalty();
        fitness.hard += dist.hard;
        fitness.soft += dist.soft * self.data.optimization.distribution;

        fitness
    }

    fn evaluate_solutions_fitness(&self, solutions: &[Solution]) -> Vec<Fitness> {
        // parallelizing this should be a change from `iter` to `par_iter`
        solutions
            .iter()
            .map(|sol| self.solution_fitness(sol))
            .collect()
    }
}
