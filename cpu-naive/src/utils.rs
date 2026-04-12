use parser::timeslots::TimeSlots;

use crate::model::RoomData;

#[inline(always)]
pub fn timeslots_overlap(a: &TimeSlots, b: &TimeSlots) -> bool {
    let shared_weeks = a.weeks.0 & b.weeks.0;
    let shared_days = a.days.0 & b.days.0;
    if shared_weeks == 0 || shared_days == 0 {
        return false;
    }

    a.start < b.start + b.length && b.start < a.start + a.length
}

pub fn travel_time_between(rooms: &[RoomData], room_a: usize, room_b: usize) -> u32 {
    if room_a == room_b {
        return 0;
    }

    rooms[room_a]
        .travels
        .iter()
        .find(|t| t.dest_room_idx == room_b)
        .map(|t| t.travel_time)
        .unwrap_or(0)
}

pub fn insufficient_travel_time(a: &TimeSlots, b: &TimeSlots, travel: u32) -> bool {
    let shared_weeks = a.weeks.0 & b.weeks.0;
    let shared_days = a.days.0 & b.days.0;
    if shared_weeks == 0 || shared_days == 0 {
        return false;
    }
    let a_end = a.start + a.length;
    let b_end = b.start + b.length;
    let gap = if a_end <= b.start {
        b.start - a_end
    } else if b_end <= a.start {
        a.start - b_end
    } else {
        return false;
    };

    gap < travel
}
