use std::collections::{HashMap, HashSet};

use crate::model::{Solution, TimetableData};

pub type ClassStudents = HashMap<usize, HashSet<usize>>;

/// returns class_idx -> list of student indices, `None` if a subpart of the
/// chosen config is too small for all students
pub fn assign_students(data: &TimetableData, solution: &Solution) -> Option<ClassStudents> {
    let mut class_students: HashMap<usize, HashSet<usize>> = HashMap::new();
    for (student_idx, student) in data.students.iter().enumerate() {
        for &course_idx in &student.course_indices {
            let course = &data.courses[course_idx];
            let course_choice = &solution.course_choices[course_idx];

            // chosen config
            let config_idx = course.configs_start + course_choice.config_offset;
            let config = &data.configs[config_idx];

            // subparts assigned to current student
            let mut assigned = vec![false; config.subparts_end - config.subparts_start];
            // last subpart id it the _leaf_, meaning it doesn't have a class with
            // a parent pointer to it
            for subpart_idx in (config.subparts_start..config.subparts_end).rev() {
                if assigned[subpart_idx - config.subparts_start] {
                    continue;
                }

                // find first non-full class
                let mut chosen_class_idx = None;

                let subpart = &data.subparts[subpart_idx];
                for class_idx in subpart.classes_start..subpart.classes_end {
                    if !class_full(data, &class_students, class_idx) {
                        chosen_class_idx = Some(class_idx);
                        break;
                    }
                }

                let Some(mut class_idx) = chosen_class_idx else {
                    return None; // all classes full
                };

                // assign to class + all ancestors
                loop {
                    if class_full(data, &class_students, class_idx) {
                        return None;
                    }

                    class_students
                        .entry(class_idx)
                        .or_default()
                        .insert(student_idx);

                    let mut class_subpart_idx = None;
                    for subpart_idx in config.subparts_start..config.subparts_end {
                        let subpart = &data.subparts[subpart_idx];
                        if (subpart.classes_start..subpart.classes_end).contains(&class_idx) {
                            class_subpart_idx = Some(subpart_idx);
                            break;
                        }
                    }
                    let class_subpart_idx = class_subpart_idx.unwrap();

                    assigned[class_subpart_idx - config.subparts_start] = true;

                    match data.classes[class_idx].parent {
                        Some(parent_idx) => class_idx = parent_idx,
                        None => break,
                    }
                }
            }
        }
    }

    // actually, all classes need to be assigned!
    // class_students.retain(|_, students| !students.is_empty());

    Some(class_students)
}

fn class_full(data: &TimetableData, class_students: &ClassStudents, class_idx: usize) -> bool {
    let class = &data.classes[class_idx];
    let current_size = class_students
        .get(&class_idx)
        .map(HashSet::len)
        .unwrap_or(0);

    match class.limit {
        Some(limit) => current_size as u32 >= limit,
        None => false,
    }
}
