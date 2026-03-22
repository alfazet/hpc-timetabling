use crate::{model::TimetableData, solution::Solution};

#[derive(Debug, Clone)]
pub struct StudentAssignments {
    /// `i`-th element corresponds to index `i` in [TimetableData::classes]
    pub classes: Vec<ClassAttendance>,
}

#[derive(Debug, Clone)]
pub struct ClassAttendance {
    /// indices into [TimetableData::students]
    pub students: Vec<usize>,
}

/// returns class_idx -> list of student indices, `None` if a subpart of the
/// chosen config is too small for all students
pub fn assign_students(data: &TimetableData, solution: &Solution) -> Option<StudentAssignments> {
    // TODO: right now we don't even need the solution but it would make sense
    // to use it somehow to get a better assignment

    // TODO: probably doesn't even work correctly...

    let mut assignments = StudentAssignments::new(data.classes.len());

    for (student_idx, student) in data.students.iter().enumerate() {
        for &course_idx in &student.course_indices {
            let course = &data.courses[course_idx];

            let mut assigned = false;
            for config_idx in course.configs_start..course.configs_end {
                if try_assign_student_to_config(data, student_idx, config_idx, &mut assignments) {
                    assigned = true;
                    break;
                }
            }

            if !assigned {
                return None;
            }
        }
    }

    Some(assignments)
}

fn try_assign_student_to_config(
    data: &TimetableData,
    student_idx: usize,
    config_idx: usize,
    assignments: &mut StudentAssignments,
) -> bool {
    let config = &data.configs[config_idx];

    let n_subparts = config.subparts_end - config.subparts_start;
    let mut assigned = vec![false; n_subparts];

    let mut chosen_classes = Vec::with_capacity(n_subparts);

    for subpart_idx in (config.subparts_start..config.subparts_end).rev() {
        let local_idx = subpart_idx - config.subparts_start;

        if assigned[local_idx] {
            continue;
        }

        let subpart = &data.subparts[subpart_idx];

        let mut found = None;

        for class_idx in subpart.classes_start..subpart.classes_end {
            if !class_full(data, assignments, class_idx) {
                found = Some(class_idx);
                break;
            }
        }

        let Some(mut class_idx) = found else {
            return false;
        };

        loop {
            if class_full(data, assignments, class_idx) {
                return false;
            }

            chosen_classes.push(class_idx);

            let subpart_idx = data.classes[class_idx].subpart_id;
            let local_idx = subpart_idx - config.subparts_start;
            assigned[local_idx] = true;

            match data.classes[class_idx].parent {
                Some(parent) => class_idx = parent,
                None => break,
            }
        }
    }

    for class_idx in chosen_classes {
        assignments.classes[class_idx].students.push(student_idx);
    }

    true
}

fn class_full(data: &TimetableData, assignments: &StudentAssignments, class_idx: usize) -> bool {
    let class = &data.classes[class_idx];
    let current_size = assignments
        .classes
        .get(class_idx)
        .map(|a| a.students.len())
        .unwrap_or(0);

    match class.limit {
        Some(limit) => current_size as u32 >= limit,
        None => false,
    }
}

impl StudentAssignments {
    pub fn new(class_count: usize) -> Self {
        Self { classes: vec![ClassAttendance::new(); class_count] }
    }
}

impl ClassAttendance {
    pub fn new() -> Self {
        Self { students: vec![] }
    }
}
