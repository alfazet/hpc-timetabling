use rand::{Rng, RngExt};

use crate::model::{RoomOption, TimeOption, TimetableData};

/// one particular assignment of classes to meetings times and rooms
#[derive(Debug, Clone, Default)]
pub struct Solution {
    /// time slot assignments
    /// `times[i]` = assignment for the `i`-th class
    pub times: Vec<TimeOption>,
    /// room assignments
    /// `rooms[i]` = assignment for the `i`-th class,
    /// None if the class doesn't require a room
    pub rooms: Vec<Option<RoomOption>>,
    /// `config_preferences[i][j]` = preferred config index for the `j`-th course of the `i`-th student
    /// (offset into `course.configs_start..course.configs_end`)
    pub config_preferences: Vec<Vec<usize>>,
}

impl Solution {
    /// generates a random (quite possibly useless) solution
    /// by assigning to each class a random time slot and a random room
    /// out of its [TimeOption]s and [RoomOption]s
    pub fn new(data: &TimetableData, rng: &mut dyn Rng) -> Self {
        let times: Vec<_> = data
            .classes
            .iter()
            .map(|class| {
                let i = rng.random_range(class.times_start..class.times_end);
                data.time_options[i].clone()
            })
            .collect();
        let rooms: Vec<_> = data
            .classes
            .iter()
            .map(|class| {
                class.needs_room().then(|| {
                    let i = rng.random_range(class.rooms_start..class.rooms_end);
                    data.room_options[i].clone()
                })
            })
            .collect();
        let config_preferences: Vec<_> = data
            .students
            .iter()
            .map(|student| {
                student
                    .course_indices
                    .iter()
                    .map(|&course_idx| {
                        let course = &data.courses[course_idx];
                        let n_configs = course.configs_end - course.configs_start;

                        rng.random_range(0..n_configs)
                    })
                    .collect()
            })
            .collect();

        Self {
            times,
            rooms,
            config_preferences,
        }
    }
}

#[cfg(test)]
mod tests {
    use parser::Problem;

    use crate::solution::*;

    fn sample_data() -> TimetableData {
        let xml = include_str!("../../data/itc2019/sample.xml");
        let problem = Problem::parse(xml).unwrap();

        TimetableData::new(problem)
    }

    #[test]
    fn random_solution_is_consistent() {
        let data = sample_data();
        let mut rng = rand::rng();
        let sol = Solution::new(&data, &mut rng);
        assert_eq!(sol.times.len(), data.classes.len());
        assert_eq!(sol.rooms.len(), data.classes.len());
    }
}
