use rand::{Rng, seq::SliceRandom};
use rayon::prelude::*;

use crate::{assigner, evaluator, model::TimetableData, penalty::Penalty, solution::Solution};

pub struct HillClimbing {
    /// number of full passes over all classes per solution
    n_passes: usize,
    /// top fraction of the population to apply local search to
    top_fraction: f32,
}

impl HillClimbing {
    pub fn new(n_passes: usize, top_fraction: f32) -> Self {
        Self {
            n_passes,
            top_fraction,
        }
    }

    pub fn optimize(
        &self,
        solutions: &mut [Solution],
        penalties: &mut [Penalty],
        data: &TimetableData,
    ) {
        let n = solutions.len();
        let n_to_improve = ((n as f32 * self.top_fraction) as usize).clamp(1, n);
        let mut indices: Vec<_> = (0..n).collect();
        indices.par_sort_unstable_by(|&a, &b| penalties[a].cmp(&penalties[b]));
        let top_indices: Vec<_> = indices[..n_to_improve].to_vec();

        let mut work_items: Vec<_> = top_indices
            .par_iter()
            .map(|&idx| (idx, solutions[idx].clone(), penalties[idx]))
            .collect();
        work_items.par_iter_mut().for_each(|(_, sol, penalty)| {
            let mut rng = rand::rng();
            Self::climb(sol, penalty, data, self.n_passes, &mut rng);
        });
        for (idx, sol, penalty) in work_items {
            solutions[idx] = sol;
            penalties[idx] = penalty;
        }
    }

    fn climb(
        sol: &mut Solution,
        penalty: &mut Penalty,
        data: &TimetableData,
        n_passes: usize,
        rng: &mut dyn Rng,
    ) {
        let n_classes = data.classes.len();
        let mut class_order: Vec<_> = (0..n_classes).collect();
        for _ in 0..n_passes {
            class_order.shuffle(rng);
            for &class_idx in &class_order {
                Self::search_time_slots(class_idx, sol, penalty, data);
                if data.classes[class_idx].needs_room() {
                    Self::search_rooms(class_idx, sol, penalty, data);
                }
            }
        }
    }

    /// tries every other time slot option for this class
    /// ends the search when any improvment is found
    fn search_time_slots(
        class_idx: usize,
        sol: &mut Solution,
        penalty: &mut Penalty,
        data: &TimetableData,
    ) {
        let class = &data.classes[class_idx];
        let original_time = sol.times[class_idx].clone();
        let mut was_improved = false;

        for time_opt_idx in class.times_start..class.times_end {
            let cand = &data.time_options[time_opt_idx];
            if cand.times == original_time.times && cand.penalty == original_time.penalty {
                continue;
            }
            sol.times[class_idx] = cand.clone();
            let assignment = assigner::assign_students(data, sol);
            let new_penalty = evaluator::evaluate(sol, data, &assignment);
            if new_penalty < *penalty {
                *penalty = new_penalty;
                was_improved = true;
                break;
            }
        }
        if !was_improved {
            sol.times[class_idx] = original_time;
        }
    }

    /// tries every other room option for this class
    /// ends the search when any improvment is found
    fn search_rooms(
        class_idx: usize,
        sol: &mut Solution,
        penalty: &mut Penalty,
        data: &TimetableData,
    ) {
        let class = &data.classes[class_idx];
        let original_room = sol.rooms[class_idx].clone();
        let mut was_improved = false;
        for room_opt_idx in class.rooms_start..class.rooms_end {
            let cand = &data.room_options[room_opt_idx];
            if let Some(ref orig) = original_room
                && cand.room_idx == orig.room_idx
                && cand.penalty == orig.penalty
            {
                continue;
            }
            sol.rooms[class_idx] = Some(cand.clone());
            let assignment = assigner::assign_students(data, sol);
            let new_penalty = evaluator::evaluate(sol, data, &assignment);
            if new_penalty < *penalty {
                *penalty = new_penalty;
                was_improved = true;
                break;
            }
        }
        if !was_improved {
            sol.rooms[class_idx] = original_room;
        }
    }
}
