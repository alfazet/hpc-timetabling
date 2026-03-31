use crate::assigner::{self, StudentAssignment};
use crate::distribution::Distribution;
use crate::{
    crossover::Crossover,
    elitism::Elitism,
    model::{RoomData, TimetableData},
    mutation::Mutation,
    penalty::Penalty,
    selection::Selection,
    solution::Solution,
};
use parser::timeslots::TimeSlots;
use rand::Rng;

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
    /// amount of consecutive current generations with no improvement in penalty
    no_improvement: usize,
    last_penalty: Option<Penalty>,
}

impl<S, C, M> Solver for NaiveSolver<S, C, M>
where
    S: Selection,
    C: Crossover,
    M: Mutation,
{
    fn solve(&mut self, rng: &mut dyn Rng) -> EvaluatedSolution {
        // because we don't even use the solution (for now) we can generate it only once
        let assignment = assigner::assign_students(&self.data);
        let mut solutions = self.initialize_solutions(rng);
        for generation in 0..self.generations {
            let mut penalties = self.evaluate_solutions_penalties(&solutions, &assignment);
            let (top_solutions, top_fitness, mut other_solutions, mut other_fitness) =
                self.elitism.split(solutions, penalties);
            let selected = self.selection.select(rng, &other_solutions, &other_fitness);
            self.crossover
                .crossover(rng, &mut other_solutions, &selected);
            self.mutation.mutate(rng, &mut other_solutions, &self.data);

            // merge unchanged top solutions with crossed-over/mutated others
            other_solutions.extend(top_solutions);
            solutions = other_solutions;
            other_fitness.extend(top_fitness);
            penalties = other_fitness;

            let min_penalty = penalties
                .into_iter()
                .min()
                .expect("solutions vec shouldn't be empty");
            eprintln!(
                "min penalty after {} generations: {}",
                generation, min_penalty
            );

            if let Some(last_penalty) = self.last_penalty
                && last_penalty == min_penalty
            {
                self.no_improvement += 1;
            } else {
                self.no_improvement = 0;
            }
            self.last_penalty = Some(min_penalty);

            self.adjust_parameters();
        }
        let final_penalty = self.evaluate_solutions_penalties(&solutions, &assignment);
        let min_idx = final_penalty
            .iter()
            .enumerate()
            .min_by(|(_, f1), (_, f2)| f1.cmp(f2))
            .expect("solutions vec shouldn't be empty")
            .0;

        EvaluatedSolution {
            inner: solutions[min_idx].clone(),
            penalty: final_penalty[min_idx],
            student_assignment: assignment,
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
        population_size: usize,
        generations: usize,
        data: TimetableData,
        elitism: Elitism,
        selection: S,
        crossover: C,
        mutation: M,
    ) -> Self {
        Self {
            population_size,
            generations,
            data,
            elitism,
            selection,
            crossover,
            mutation,
            no_improvement: 0,
            last_penalty: None,
        }
    }

    fn initialize_solutions(&mut self, rng: &mut dyn Rng) -> Vec<Solution> {
        let mut solutions = Vec::with_capacity(self.population_size);
        for _ in 0..self.population_size {
            solutions.push(Solution::new(&self.data, rng));
        }

        solutions
    }

    fn adjust_parameters(&mut self) {
        let max_no_improvement = 30;
        if self.no_improvement < max_no_improvement {
            return;
        }
        eprintln!(
            "no improvement for {} generations, adjusting parameters...",
            max_no_improvement
        );

        let p = self.mutation.probability();
        eprint!("mutation: {p} ->");
        *p = (*p * 1.2).min(0.5);
        eprintln!(" {p}");

        let p = self.crossover.probability();
        eprint!("crossover: {p} ->");
        *p = (*p * 1.2).min(1.0);
        eprintln!(" {p}");

        self.no_improvement = 0;
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

    fn student_assignment_conflicts(&self, sol: &Solution, assignment: &StudentAssignment) -> u32 {
        let mut n_conflicts = 0;
        let mut classes_per_student = vec![Vec::new(); self.data.students.len()];
        for (class_idx, student_list) in assignment.students_in_classes.iter().enumerate() {
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

    fn classes_hard_penalties(&self, sol: &Solution, assignment: &StudentAssignment) -> u32 {
        let mut n_violations = 0;

        n_violations += self.classes_student_limits_penalty(assignment);
        n_violations += self.students_not_enrolled_in_exactly_one_per_subpart(assignment);
        n_violations += self.students_not_enrolled_in_parent_penalty(assignment);
        n_violations += self.rooms_capacity_limits_penalty(sol, assignment);
        n_violations += self.classes_in_unavailable_rooms_penalty(sol);
        n_violations += self.time_intervals_overlap_penalty(sol);

        n_violations
    }

    /// counts the hard violations for students enrolled in multiple courses from
    /// a subpart
    fn students_not_enrolled_in_exactly_one_per_subpart(
        &self,
        assignment: &StudentAssignment,
    ) -> u32 {
        let mut n_violations = 0u32;

        for (student_idx, student) in self.data.students.iter().enumerate() {
            for &course_idx in &student.course_indices {
                let course = &self.data.courses[course_idx];

                let mut best_config_penalty = u32::MAX;

                for config_idx in course.configs_start..course.configs_end {
                    let config = &self.data.configs[config_idx];

                    let mut penalty = 0u32;

                    for subpart_idx in config.subparts_start..config.subparts_end {
                        let subpart = &self.data.subparts[subpart_idx];

                        let assigned = (subpart.classes_start..subpart.classes_end)
                            .filter(|&class_idx| {
                                assignment.students_in_classes[class_idx].contains(&student_idx)
                            })
                            .count();

                        // should be assigned to exaclty one
                        if assigned == 0 {
                            penalty += 1;
                        } else if assigned > 1 {
                            penalty += (assigned as u32) - 1;
                        }
                    }

                    // since the student should attend just one config, we pick
                    // the one with lowest penalty as the "indended" solution
                    best_config_penalty = best_config_penalty.min(penalty);
                }

                debug_assert!(
                    best_config_penalty != u32::MAX,
                    "a course should have at least one subpart"
                );
                n_violations += best_config_penalty;
            }
        }

        n_violations
    }

    /// counts the hard violations for students not enrolled in a parent of a
    /// class they're attending
    fn students_not_enrolled_in_parent_penalty(&self, assignment: &StudentAssignment) -> u32 {
        let mut n_violations = 0;
        for (class_idx, class) in self.data.classes.iter().enumerate() {
            let Some(parent) = class.parent else {
                continue;
            };

            for stud_idx in &assignment.students_in_classes[class_idx] {
                if !assignment.students_in_classes[parent].contains(stud_idx) {
                    n_violations += 1;
                }
            }
        }

        n_violations
    }

    /// counts the hard violations for classes having more students
    /// than allowed by their limit
    fn classes_student_limits_penalty(&self, assignment: &StudentAssignment) -> u32 {
        assignment
            .students_in_classes
            .iter()
            .enumerate()
            .map(|(index, class)| {
                if let Some(limit) = self.data.classes[index].limit
                    && class.len() > limit as usize
                {
                    return 1;
                }
                0
            })
            .sum()
    }

    /// counts the hard violations for classes taking place
    /// in rooms that don't have enough capacity
    fn rooms_capacity_limits_penalty(&self, sol: &Solution, assignment: &StudentAssignment) -> u32 {
        assignment
            .students_in_classes
            .iter()
            .enumerate()
            .map(|(index, class)| {
                if let Some(room_option) = &sol.rooms[index]
                    && self.data.rooms[room_option.room_idx].capacity < class.len() as u32
                {
                    return 1;
                }
                0
            })
            .sum()
    }

    /// counts the hard violations for classes taking place
    /// in rooms that are unavailable in chosen timeslots
    fn classes_in_unavailable_rooms_penalty(&self, sol: &Solution) -> u32 {
        sol.times
            .iter()
            .enumerate()
            .map(|(index, time_option)| {
                if let Some(room_option) = &sol.rooms[index] {
                    let unavailabilities = &self.data.rooms[room_option.room_idx].unavailabilities;
                    let times = &time_option.times;
                    if unavailabilities
                        .iter()
                        .any(|unavailability| Self::timeslots_overlap(unavailability, times))
                    {
                        return 1;
                    }
                }
                0
            })
            .sum()
    }

    /// counts the hard violations -- time intervals of two
    /// classes overlap in the same room
    fn time_intervals_overlap_penalty(&self, sol: &Solution) -> u32 {
        sol.rooms
            .iter()
            .enumerate()
            .map(|(index, room_option)| {
                if let Some(room_idx) = room_option.as_ref().map(|r| r.room_idx) {
                    for i in index + 1..sol.rooms.len() {
                        if let Some(i_room_idx) = sol.rooms[i].as_ref().map(|r| r.room_idx)
                            && room_idx == i_room_idx
                            && Self::timeslots_overlap(&sol.times[index].times, &sol.times[i].times)
                        {
                            return 1;
                        }
                    }
                }
                0
            })
            .sum()
    }

    fn rooms_penalty(&self, sol: &Solution) -> u32 {
        sol.rooms.iter().flatten().map(|r| r.penalty).sum()
    }

    fn times_penalty(&self, sol: &Solution) -> u32 {
        sol.times.iter().map(|t| t.penalty).sum()
    }

    fn solution_penalty(&self, sol: &Solution, assignment: &StudentAssignment) -> Penalty {
        let mut penalty = Penalty::new();

        let clas = self.classes_hard_penalties(sol, assignment);
        penalty.hard += clas;

        let stud = self.student_assignment_conflicts(sol, assignment);
        penalty.soft += stud * self.data.optimization.student;

        let room = self.rooms_penalty(sol);
        penalty.soft += room * self.data.optimization.room;

        let time = self.times_penalty(sol);
        penalty.soft += time * self.data.optimization.time;

        let dist = Distribution::new(&self.data, sol).calculate_penalty();
        penalty.hard += dist.hard;
        penalty.soft += dist.soft * self.data.optimization.distribution;

        penalty
    }

    fn evaluate_solutions_penalties(
        &self,
        solutions: &[Solution],
        assignment: &StudentAssignment,
    ) -> Vec<Penalty> {
        // parallelizing this should be a change from `iter` to `par_iter`
        solutions
            .iter()
            .map(|sol| self.solution_penalty(sol, assignment))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use parser::Problem;

    use crate::{
        assigner::StudentAssignment, crossover::OnePointCrossover, elitism::Elitism,
        model::TimetableData, mutation::BasicMutation, selection::TournamentSelection,
        solver::NaiveSolver,
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
            OnePointCrossover::new(),
            BasicMutation::new(0.0),
        );
        solver
    }

    #[test]
    fn test_students_not_enrolled_in_parent_penalty_empty() {
        let solver = solver();
        let assignment = StudentAssignment {
            students_in_classes: vec![vec![]; 3],
        };

        let penalty = solver.students_not_enrolled_in_parent_penalty(&assignment);

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
}
