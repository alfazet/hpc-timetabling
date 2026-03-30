use crate::model::TimetableData;

pub struct StudentAssignment {
    /// students_in_classes[i] = indices of students taking the class with index `i`
    pub students_in_classes: Vec<Vec<usize>>,
}

pub fn assign_students(data: &TimetableData) -> StudentAssignment {
    // not using the solution for now
    let mut students_in_classes = vec![Vec::new(); data.classes.len()];
    for (student_idx, student) in data.students.iter().enumerate() {
        for &course_idx in &student.course_indices {
            let course = &data.courses[course_idx];
            'config_loop: for config_idx in course.configs_start..course.configs_end {
                let config = &data.configs[config_idx];
                let mut class_taken_in_subpart =
                    vec![None; config.subparts_end - config.subparts_start];
                for subpart_idx in config.subparts_start..config.subparts_end {
                    let subpart = &data.subparts[subpart_idx];
                    let mut assigned_subpart = false;
                    'candidate_loop: for class_idx in subpart.classes_start..subpart.classes_end {
                        let mut local_assignment = class_taken_in_subpart.clone();
                        let mut cur_idx = class_idx;
                        loop {
                            let subpart_idx = data.classes[cur_idx].subpart_idx;
                            local_assignment[subpart_idx - config.subparts_start] = Some(cur_idx);
                            match data.classes[cur_idx].parent {
                                Some(p) => cur_idx = p,
                                None => break,
                            }
                        }

                        // check capacity for every class we'd be adding the student to
                        let mut ok = true;
                        for &opt_c in local_assignment.iter().flatten() {
                            if let Some(limit) = data.classes[opt_c].limit {
                                let current = students_in_classes[opt_c].len() as u32;
                                if current + 1 > limit {
                                    ok = false;
                                    break;
                                }
                            }
                        }
                        if ok {
                            class_taken_in_subpart = local_assignment;
                            assigned_subpart = true;
                            break 'candidate_loop;
                        }
                    }
                    if !assigned_subpart {
                        // we have to try out another config if we couldn't
                        // find a single fitting class in a subpart contained within that config
                        continue 'config_loop;
                    }
                }

                // all subparts assigned
                for i in &class_taken_in_subpart {
                    let c = i.expect("all subparts should be assigned");
                    students_in_classes[c].push(student_idx);
                }
                break;
            }
        }
    }

    StudentAssignment {
        students_in_classes,
    }
}
