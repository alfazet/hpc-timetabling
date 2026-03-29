use rand::{Rng, RngExt, seq::SliceRandom};

use crate::model::{RoomOption, TimeOption, TimetableData};

/// one particular assignment of classes to meetings times and rooms
#[derive(Debug, Clone)]
pub struct Solution {
    /// time slot assignments
    /// `times[i]` = assignment for the `i`-th class
    pub times: Vec<TimeOption>,
    /// room assignments
    /// `rooms[i]` = assignment for the `i`-th class,
    /// None if the class doesn't require a room
    pub rooms: Vec<Option<RoomOption>>,
    /// student assignment
    /// `students[i]` = vec of indices of students who will attend the `i`-th class
    pub students_in_classes: Vec<Vec<usize>>,
}

impl Solution {
    /// generates a random (quite possibly useless) solution
    /// by assigning to each class a random time slot and a random room
    /// out of its [TimeOption]s and [RoomOption]s
    ///
    /// also assigns students randomly, respecting class limits
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

        Self {
            times,
            rooms,
            students_in_classes: Self::assign_students(data, rng),
        }
    }

    fn assign_students(data: &TimetableData, rng: &mut impl Rng) -> Vec<Vec<usize>> {
        let mut students_in_classes = vec![vec![]; data.classes.len()];
        for (student_idx, student) in data.students.iter().enumerate() {
            for &course_idx in &student.course_indices {
                let course = &data.courses[course_idx];
                let mut configs: Vec<_> = (course.configs_start..course.configs_end).collect();
                configs.shuffle(rng);
                'config_loop: for config_idx in configs {
                    let config = &data.configs[config_idx];
                    // final per-subpart decision
                    let mut class_taken_in_subpart =
                        vec![None; config.subparts_end - config.subparts_start];
                    // try to assign each subpart
                    for subpart_idx in config.subparts_start..config.subparts_end {
                        let subpart = &data.subparts[subpart_idx];
                        let mut candidates: Vec<_> =
                            (subpart.classes_start..subpart.classes_end).collect();
                        candidates.shuffle(rng);
                        let mut assigned = false;
                        'candidate_loop: for &class_idx_start in &candidates {
                            // local assignment induced by this choice
                            let mut local_assignment = class_taken_in_subpart.clone();
                            let mut class_idx = class_idx_start;
                            // propagate up the parent chain
                            loop {
                                let sp_idx = data.classes[class_idx].subpart_idx;
                                let local_idx = sp_idx - config.subparts_start;
                                local_assignment[local_idx] = Some(class_idx);
                                match data.classes[class_idx].parent {
                                    Some(p) => class_idx = p,
                                    None => break,
                                }
                            }
                            // capacity check (consider current + local assignment)
                            let mut ok = true;
                            for &opt_c in local_assignment.iter().flatten() {
                                let current = students_in_classes[opt_c].len();
                                let extra = local_assignment
                                    .iter()
                                    .flatten()
                                    .filter(|&&c| c == opt_c)
                                    .count();

                                if current + extra > data.classes[opt_c].limit.unwrap_or(u32::MAX) as usize
                                {
                                    ok = false;
                                    break;
                                }
                            }
                            if ok {
                                class_taken_in_subpart = local_assignment;
                                assigned = true;
                                break 'candidate_loop;
                            }
                        }
                        if !assigned {
                            continue 'config_loop;
                        }
                    }
                    for subpart_idx in config.subparts_start..config.subparts_end {
                        let c = class_taken_in_subpart[subpart_idx - config.subparts_start]
                            .expect("all subparts should be assigned");

                        students_in_classes[c].push(student_idx);
                    }
                    break; // success for this course
                }
            }
        }

        students_in_classes
    }
}

#[cfg(test)]
mod tests {
    use parser::Problem;
    use rand::{SeedableRng, rngs::StdRng};

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

    fn setup() -> (TimetableData, StdRng) {
        // a small instance
        let xml = include_str!("../../data/itc2019/with-students/wbg-fal10.xml");
        let problem = Problem::parse(xml).unwrap();
        (TimetableData::new(problem), StdRng::seed_from_u64(2137))
    }

    #[test]
    fn does_not_exceed_capacity() {
        let (data, mut rng) = setup();

        let result = Solution::assign_students(&data, &mut rng);

        for (i, students) in result.iter().enumerate() {
            assert!(
                students.len() <= data.classes[i].limit.unwrap_or(u32::MAX) as usize,
                "class {} exceeded capacity",
                i
            );
        }
    }

    #[test]
    fn one_class_per_subpart() {
        let (data, mut rng) = setup();

        let result = Solution::assign_students(&data, &mut rng);

        for (student_idx, student) in data.students.iter().enumerate() {
            for &course_idx in &student.course_indices {
                let course = &data.courses[course_idx];

                for config_idx in course.configs_start..course.configs_end {
                    let config = &data.configs[config_idx];

                    for subpart_idx in config.subparts_start..config.subparts_end {
                        let count = result
                            .iter()
                            .enumerate()
                            .filter(|(class_idx, students)| {
                                students.contains(&student_idx)
                                    && data.classes[*class_idx].subpart_idx == subpart_idx
                            })
                            .count();

                        assert!(count <= 1, "multiple classes in subpart");
                    }
                }
            }
        }
    }

    #[test]
    fn respects_parent_constraints() {
        let (data, mut rng) = setup();

        let result = Solution::assign_students(&data, &mut rng);

        for (class_idx, students) in result.iter().enumerate() {
            if let Some(parent) = data.classes[class_idx].parent {
                for s in students {
                    assert!(result[parent].contains(s), "student missing parent class");
                }
            }
        }
    }
}
