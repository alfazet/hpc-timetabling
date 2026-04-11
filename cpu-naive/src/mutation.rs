use rand::{Rng, RngExt};

use crate::{model::TimetableData, solution::Solution};

pub trait Mutation {
    fn mutate(&mut self, rng: &mut dyn Rng, solutions: &mut [Solution], data: &TimetableData);
    fn probability(&mut self) -> &mut f64;
}

pub struct BasicMutation {
    probability: f64,
}

impl BasicMutation {
    pub fn new(probability: f64) -> Self {
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
            for (student_idx, student) in data.students.iter().enumerate() {
                for (i, course_idx) in student.course_indices.iter().enumerate() {
                    if rng.random_range(0.0..1.0) <= self.probability {
                        let course = &data.courses[*course_idx];
                        let n_configs = course.configs_end - course.configs_start;
                        sol.config_preferences[student_idx][i] = rng.random_range(0..n_configs);
                    }
                }
            }
        }
    }

    fn probability(&mut self) -> &mut f64 {
        &mut self.probability
    }
}
