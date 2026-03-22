use serializer::output::{Class, Output, Student};

use crate::{assigner::assign_students, model::TimetableData, solution::Solution};

pub fn output(solution: &Solution, data: &TimetableData) -> Option<Output> {
    let class_students = assign_students(data, solution)?;

    let mut classes_out = Vec::with_capacity(data.classes.len());

    for class_idx in 0..data.classes.len() {
        let class_data = &data.classes[class_idx];
        let room = &solution.rooms[class_idx];
        let time = &solution.times[class_idx];

        let (days, weeks, start) = (time.times.days, time.times.weeks, time.times.start);

        let room = room.as_ref().map(|r| data.rooms[r.room_idx].id);

        // resolve students
        let students_ids = class_students.classes[class_idx]
            .students
            .iter()
            .map(|&s_idx| Student {
                id: data.students[s_idx].id
            })
            .collect();

        classes_out.push(Class {
            id: class_data.id,
            days,
            weeks,
            start,
            room,
            students: students_ids,
        });
    }

    Some(Output {
        classes: classes_out,
    })
    */

    None
}
