use rayon::prelude::*;

use crate::{
    assigner::StudentAssignment, distribution::Distribution, model::TimetableData,
    penalty::Penalty, solution::Solution, utils,
};

pub fn evaluate(sol: &Solution, data: &TimetableData, assignment: &StudentAssignment) -> Penalty {
    let mut penalty = Penalty::new();

    let clas = classes_hard_penalties(sol, data, assignment);
    penalty.hard += clas;

    let stud = student_assignment_conflicts(sol, data, assignment);
    penalty.soft += stud * data.optimization.student;

    let room = rooms_penalty(sol);
    penalty.soft += room * data.optimization.room;

    let time = times_penalty(sol);
    penalty.soft += time * data.optimization.time;

    let dist = Distribution::new(data, sol).calculate_penalty();
    penalty.hard += dist.hard;
    penalty.soft += dist.soft * data.optimization.distribution;

    penalty
}

fn student_assignment_conflicts(
    sol: &Solution,
    data: &TimetableData,
    assignment: &StudentAssignment,
) -> u32 {
    let mut n_conflicts = 0;
    let mut classes_per_student = vec![Vec::new(); data.students.len()];
    for (class_idx, student_list) in assignment.students_in_classes.iter().enumerate() {
        for &student_idx in student_list {
            classes_per_student[student_idx].push(class_idx);
        }
    }
    for student_classes in &classes_per_student {
        for i in 0..student_classes.len() {
            for j in (i + 1)..student_classes.len() {
                let ci = student_classes[i];
                let cj = student_classes[j];
                let time_a = &sol.times[ci].times;
                let time_b = &sol.times[cj].times;
                if utils::timeslots_overlap(time_a, time_b) {
                    n_conflicts += 1;
                } else {
                    let travel = match (&sol.rooms[ci], &sol.rooms[cj]) {
                        (Some(room_a), Some(room_b)) => utils::travel_time_between(
                            &data.rooms,
                            room_a.room_idx,
                            room_b.room_idx,
                        ),
                        _ => 0,
                    };
                    if travel > 0 && utils::insufficient_travel_time(time_a, time_b, travel) {
                        n_conflicts += 1;
                    }
                }
            }
        }
    }

    n_conflicts
}

fn classes_hard_penalties(
    sol: &Solution,
    data: &TimetableData,
    assignment: &StudentAssignment,
) -> u32 {
    let mut n_violations = 0;

    n_violations += classes_student_limits_penalty(data, assignment);
    n_violations += students_not_enrolled_in_exactly_one_per_subpart(data, assignment);
    n_violations += students_not_enrolled_in_parent_penalty(data, assignment);
    n_violations += rooms_capacity_limits_penalty(sol, data, assignment);
    n_violations += classes_in_unavailable_rooms_penalty(sol, data);
    n_violations += time_intervals_overlap_penalty(sol);

    n_violations
}

/// counts the hard violations for students enrolled in multiple courses from
/// a subpart
fn students_not_enrolled_in_exactly_one_per_subpart(
    data: &TimetableData,
    assignment: &StudentAssignment,
) -> u32 {
    let mut n_violations = 0u32;

    for (student_idx, student) in data.students.iter().enumerate() {
        for &course_idx in &student.course_indices {
            let course = &data.courses[course_idx];

            let mut best_config_penalty = u32::MAX;

            for config_idx in course.configs_start..course.configs_end {
                let config = &data.configs[config_idx];

                let mut penalty = 0u32;

                for subpart_idx in config.subparts_start..config.subparts_end {
                    let subpart = &data.subparts[subpart_idx];

                    let assigned = (subpart.classes_start..subpart.classes_end)
                        .filter(|&class_idx| {
                            assignment.students_in_classes[class_idx].contains(&student_idx)
                        })
                        .count();

                    // should be assigned to exactly one
                    if assigned == 0 {
                        penalty += 1;
                    } else if assigned > 1 {
                        penalty += (assigned as u32) - 1;
                    }
                }

                // since the student should attend just one config, we pick
                // the one with lowest penalty as the "indended" solution
                best_config_penalty = best_config_penalty.min(penalty);
            }

            debug_assert!(
                best_config_penalty != u32::MAX,
                "a course should have at least one subpart"
            );
            n_violations += best_config_penalty;
        }
    }

    n_violations
}

/// counts the hard violations for students not enrolled in a parent of a
/// class they're attending
fn students_not_enrolled_in_parent_penalty(
    data: &TimetableData,
    assignment: &StudentAssignment,
) -> u32 {
    let mut n_violations = 0;
    for (class_idx, class) in data.classes.iter().enumerate() {
        let Some(parent) = class.parent else {
            continue;
        };

        for stud_idx in &assignment.students_in_classes[class_idx] {
            if !assignment.students_in_classes[parent].contains(stud_idx) {
                n_violations += 1;
            }
        }
    }

    n_violations
}

/// counts the hard violations for classes having more students
/// than allowed by their limit
fn classes_student_limits_penalty(data: &TimetableData, assignment: &StudentAssignment) -> u32 {
    assignment
        .students_in_classes
        .iter()
        .enumerate()
        .map(|(index, class)| {
            if let Some(limit) = data.classes[index].limit
                && class.len() > limit as usize
            {
                return 1;
            }
            0
        })
        .sum()
}

/// counts the hard violations for classes taking place
/// in rooms that don't have enough capacity
fn rooms_capacity_limits_penalty(
    sol: &Solution,
    data: &TimetableData,
    assignment: &StudentAssignment,
) -> u32 {
    assignment
        .students_in_classes
        .iter()
        .enumerate()
        .map(|(index, class)| {
            if let Some(room_option) = &sol.rooms[index]
                && data.rooms[room_option.room_idx].capacity < class.len() as u32
            {
                1
            } else {
                0
            }
        })
        .sum()
}

/// counts the hard violations for classes taking place
/// in rooms that are unavailable in chosen timeslots
fn classes_in_unavailable_rooms_penalty(sol: &Solution, data: &TimetableData) -> u32 {
    sol.times
        .iter()
        .enumerate()
        .map(|(index, time_option)| {
            if let Some(room_option) = &sol.rooms[index] {
                let unavailabilities = &data.rooms[room_option.room_idx].unavailabilities;
                let times = &time_option.times;
                if unavailabilities
                    .iter()
                    .any(|unavailability| utils::timeslots_overlap(unavailability, times))
                {
                    return 1;
                }
            }
            0
        })
        .sum()
}

/// counts the hard violations -- time intervals of two
/// classes overlap in the same room
fn time_intervals_overlap_penalty(sol: &Solution) -> u32 {
    let mut classes_per_room = vec![vec![]; sol.rooms.len()];
    for (i, r) in sol.rooms.iter().enumerate() {
        if let Some(room_idx) = r.as_ref().map(|r| r.room_idx) {
            classes_per_room[room_idx].push(i);
        }
    }

    let mut res = 0;
    for room_idxs in classes_per_room {
        for i in 0..room_idxs.len() {
            let times_i = &sol.times[room_idxs[i]].times;
            for j in i + 1..room_idxs.len() {
                let times_j = &sol.times[room_idxs[j]].times;
                if utils::timeslots_overlap(times_i, times_j) {
                    res += 1;
                }
            }
        }
    }
    res
}

fn rooms_penalty(sol: &Solution) -> u32 {
    sol.rooms.iter().flatten().map(|r| r.penalty).sum()
}

fn times_penalty(sol: &Solution) -> u32 {
    sol.times.iter().map(|t| t.penalty).sum()
}
