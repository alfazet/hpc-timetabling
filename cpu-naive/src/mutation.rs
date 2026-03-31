use rand::{Rng, RngExt};

use crate::{model::TimetableData, solution::Solution};

pub trait Mutation {
    fn mutate(&mut self, rng: &mut dyn Rng, solutions: &mut [Solution], data: &TimetableData);
}

pub struct BasicMutation {
    probability: f32,
}

impl BasicMutation {
    pub fn new(probability: f32) -> Self {
        Self { probability }
    }
}

impl Mutation for BasicMutation {
    fn mutate(&mut self, rng: &mut dyn Rng, solutions: &mut [Solution], data: &TimetableData) {
        let n_classes = data.classes.len();
        for sol in solutions.iter_mut() {
            for class_idx in 0..n_classes {
                if rng.random_range(0.0..1.0) <= self.probability {
                    let class = &data.classes[class_idx];
                    let time_range = class.times_start..class.times_end;
                    if !time_range.is_empty() {
                        let new_time_idx = rng.random_range(time_range);
                        sol.times[class_idx] = data.time_options[new_time_idx].clone();
                    }
                    if class.needs_room() {
                        let room_range = class.rooms_start..class.rooms_end;
                        let new_room_idx = rng.random_range(room_range);
                        sol.rooms[class_idx] = Some(data.room_options[new_room_idx].clone());
                    }
                }
            }
            // TODO: mutations on students
        }
    }
}
