use parser::timeslots::TimeSlots;

pub fn timeslots_overlap(a: &TimeSlots, b: &TimeSlots) -> bool {
    let shared_weeks = a.weeks.0 & b.weeks.0;
    let shared_days = a.days.0 & b.days.0;
    if shared_weeks == 0 || shared_days == 0 {
        return false;
    }

    a.start < b.start + b.length && b.start < a.start + a.length
}
