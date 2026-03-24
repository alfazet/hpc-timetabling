use serializer::output::{Class, Output, Student};

use crate::{model::TimetableData, solution::Solution};

pub fn output(solution: &Solution, data: &TimetableData) -> Option<Output> {
    let mut classes_out = Vec::with_capacity(data.classes.len());

    for class_idx in 0..data.classes.len() {
        let class_data = &data.classes[class_idx];
        let room = &solution.rooms[class_idx];
        let time = &solution.times[class_idx];
        let (days, weeks, start) = (time.times.days, time.times.weeks, time.times.start);
        let room = room.as_ref().map(|r| data.rooms[r.room_idx].id);

        let students_in_this_class = solution.students_in_classes[class_idx].iter().map(|&idx| Student { id: data.students[idx].id }).collect();

        let class = Class { id: class_data.id, days, weeks, start, room, students: students_in_this_class };
        classes_out.push(class);
    }

    Some(Output {
        classes: classes_out,
    })
}
