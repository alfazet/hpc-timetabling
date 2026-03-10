use quick_xml::events::BytesStart;

use crate::{ParseError, days::Days, utils::parse_value, weeks::Weeks};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rooms(Vec<Room>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoomId(i32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Travel {
    /// travel destination
    pub room: RoomId,
    /// travel time in time slots (see [crate::Problem::slots_per_day])
    /// for itc2019 - only non-zero values and symmetrical
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Unavailable {
    /// start time (from the beginning of a day) in time slots
    /// (see [crate::Problem::slots_per_day])
    pub start: u32,
    /// time length in time slots (see [crate::Problem::slots_per_day])
    pub length: u32,
    /// days on which the [Room] is NOT available
    pub days: Days,
    /// weeks on which the [Room] is NOT available
    pub weeks: Weeks,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Room {
    pub id: RoomId,
    /// max number of students
    pub capacity: u32,
    pub travels: Vec<Travel>,
    /// times when the room is unavailable
    pub unavailabilities: Vec<Unavailable>,
}

impl Room {
    pub(crate) fn parse(e: &BytesStart) -> Result<Self, ParseError> {
        todo!()
    }
}
