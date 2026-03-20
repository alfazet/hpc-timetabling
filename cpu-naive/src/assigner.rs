use std::collections::HashMap;

use crate::model::{Solution, TimetableData};

/// returns class_idx -> list of student indices, `None` if a subpart of the
/// chosen config is too small for all students
pub fn assign_students(
    data: &TimetableData,
    solution: &Solution,
) -> Option<HashMap<usize, Vec<usize>>> {
    let mut class_students: HashMap<usize, Vec<usize>> = HashMap::new();
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
                    let class = &data.classes[class_idx];
                    let current_size = class_students.get(&class_idx).map(|v| v.len()).unwrap_or(0);

                    let is_full = match class.limit {
                        Some(limit) => current_size as u32 >= limit,
                        None => false,
                    };

                    if !is_full {
                        chosen_class_idx = Some(class_idx);
                        break;
                    }
                }

                let Some(mut class_idx) = chosen_class_idx else {
                    return None; // all classes full
                };

                // assign to class + all ancestors
                loop {
                    class_students
                        .entry(class_idx)
                        .or_default()
                        .push(student_idx);

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

    class_students.retain(|_, students| !students.is_empty());

    Some(class_students)
}
