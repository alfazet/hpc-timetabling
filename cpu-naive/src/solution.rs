use rand::{Rng, RngExt};

use crate::model::{RoomOption, TimeOption, TimetableData};

/// one particular assignment of classes to meetings times and rooms
#[derive(Debug, Clone)]
pub struct Solution {
    /// time slot assignments
    /// times[i] = assignment for the i-th class
    pub times: Vec<TimeOption>,
    /// room assignments
    /// rooms[i] = assignment for the i-th class,
    /// None if the class doesn't require a room
    pub rooms: Vec<Option<RoomOption>>,
    /// student assignment
    /// students[i] = vec of indices of students who will attend the i-th class
    pub students_in_classes: Vec<Vec<usize>>,
}

impl Solution {
    /// generates a random (quite possibly useless) solution
    /// by assigning to each class a random time slot and a random room
    /// out of its TimeOptions and RoomOptions
    /// and chooses a random config for every course they want to attend
    pub fn new(data: &TimetableData, rng: &mut impl Rng) -> Self {
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

        let mut students_in_classes = vec![Vec::new(); data.classes.len()];
        for (student_idx, student) in data.students.iter().enumerate() {
            for course in student.course_indices.iter().map(|i| &data.courses[*i]) {
                let config_idx = rng.random_range(course.configs_start..course.configs_end);
                let config = &data.configs[config_idx];
                for subpart_idx in config.subparts_start..config.subparts_end {
                    let subpart = &data.subparts[subpart_idx];
                    let mut class_idx =
                        rng.random_range(subpart.classes_start..subpart.classes_end);
                    let mut class = &data.classes[class_idx];
                    // assign this student to this class and all of its ancestors
                    // ISSUE: this can add a student to a class that is an alternative to another
                    // one that they're already taking within the same subpart
                    // POSSIBLE FIX?: save which class was chosen in each subpart (in a map)
                    // and then if we're forced to take a class in subpart X (because it's a parent of some
                    // other one), we remove this student from the other class of subpart X they
                    // were assigned to previously
                    loop {
                        students_in_classes[class_idx].push(student_idx);
                        match class.parent {
                            Some(parent_idx) => {
                                class_idx = parent_idx;
                                class = &data.classes[class_idx];
                            }
                            None => break,
                        }
                    }
                }
            }
        }

        Self {
            times,
            rooms,
            students_in_classes,
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
