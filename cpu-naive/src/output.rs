use std::collections::HashMap;

use parser::{courses::ClassId, days::Days, rooms::RoomId, students::StudentId, weeks::Weeks};

use crate::{
    assigner::assign_students,
    model::{Solution, TimetableData},
};

#[derive(Debug, Clone, PartialEq)]
pub struct OutputMetadata {
    name: String,
    runtime: f32,
    cores: usize,
    technique: String,
    author: String,
    institution: String,
    country: String,
}

impl Default for OutputMetadata {
    fn default() -> Self {
        Self {
            name: "name".into(),
            runtime: 0.0,
            cores: 1,
            technique: "Genetic Algorithm".into(),
            author: "todo".into(),
            institution: "todo".into(),
            country: "todo".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    id: ClassId,
    days: Days,
    weeks: Weeks,
    start: u32,
    room: Option<RoomId>,
    students: Vec<StudentId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    metadata: OutputMetadata,
    classes: Vec<Class>,
}

impl Output {
    pub fn new(solution: Solution, data: TimetableData) -> Self {
        let class_students = assign_students(&data, &solution).unwrap();

        // helper: class_idx -> (time_option_idx, room_option_idx)
        let mut class_assignment: HashMap<usize, (usize, Option<usize>)> = HashMap::new();

        // extract chosen time/room for each class from solution
        for (course_idx, course_choice) in solution.course_choices.iter().enumerate() {
            let course = &data.courses[course_idx];
            let config_idx = course.configs_start + course_choice.config_offset;
            let config = &data.configs[config_idx];

            for (subpart_offset, subpart_idx) in
                (config.subparts_start..config.subparts_end).enumerate()
            {
                let subpart_choice = &course_choice.subpart_choices[subpart_offset];
                let subpart = &data.subparts[subpart_idx];

                for i in 0..subpart_choice.class_offset.len() {
                    let class_idx = subpart.classes_start + subpart_choice.class_offset[i];

                    let time_idx =
                        data.classes[class_idx].times_start + subpart_choice.time_offset[i];

                    let room_idx = subpart_choice.room_offset[i]
                        .map(|r| data.classes[class_idx].rooms_start + r);

                    class_assignment.insert(class_idx, (time_idx, room_idx));
                }
            }
        }

        let mut classes_out = Vec::new();

        for (class_idx, students) in class_students {
            let class_data = &data.classes[class_idx];

            let Some(&(time_option_idx, room_option_idx)) = class_assignment.get(&class_idx) else {
                continue; // class not used in solution
            };

            let time = &data.time_options[time_option_idx];

            let (days, weeks, start) = (
                time.times.days.clone(),
                time.times.weeks.clone(),
                time.times.start,
            );

            let room = room_option_idx.map(|r| {
                let idx = data.room_options[r].room_idx;
                RoomId(data.rooms[idx].id)
            });

            let students_ids = students
                .iter()
                .map(|&s_idx| StudentId(data.students[s_idx].id))
                .collect();

            classes_out.push(Class {
                id: ClassId(class_data.original_id),
                days,
                weeks,
                start,
                room,
                students: students_ids,
            });
        }

        Output {
            metadata: OutputMetadata::default(),
            classes: classes_out,
        }
    }
}
