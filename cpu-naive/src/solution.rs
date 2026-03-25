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
                // indexing into this vec is shifted by config.subparts_start
                let mut class_taken_in_subpart =
                    vec![0; config.subparts_end - config.subparts_start];
                for subpart_idx in config.subparts_start..config.subparts_end {
                    let subpart = &data.subparts[subpart_idx];
                    let mut class_idx =
                        rng.random_range(subpart.classes_start..subpart.classes_end);
                    let mut class = &data.classes[class_idx];
                    // assign this student to this class and all of its ancestors
                    // traversal_... because this idx changes as we traverse the dependency tree
                    let mut traversal_subpart_idx = subpart_idx;
                    loop {
                        // this might overwrite a previously taken class with another one from the same
                        // subpart to satisfy the parent constraint
                        class_taken_in_subpart[traversal_subpart_idx - config.subparts_start] =
                            class_idx;
                        match class.parent {
                            Some(parent_idx) => {
                                class_idx = parent_idx;
                                class = &data.classes[class_idx];
                                traversal_subpart_idx = class.subpart_idx;
                            }
                            None => break,
                        }
                    }
                }
                for subpart_idx in config.subparts_start..config.subparts_end {
                    // eprintln!(
                    //     "resolving subpart {:?} for student {:?}",
                    //     data.subparts[subpart_idx].id, student.id
                    // );
                    let taken_class_idx =
                        class_taken_in_subpart[subpart_idx - config.subparts_start];
                    // eprintln!(
                    //     "taken class from this subpart: {:?}",
                    //     data.classes[taken_class_idx].id
                    // );
                    students_in_classes[taken_class_idx].push(student_idx);
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
