use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

use crate::{ParseError, timeslots::TimeSlots, utils::{define_id, parse_value}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rooms(pub Vec<Room>);

define_id!(RoomId);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Travel {
    /// travel destination
    pub room: RoomId,
    /// travel time in time slots (see [crate::Problem::slots_per_day])
    /// for itc2019 - only non-zero values and symmetrical
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Room {
    pub id: RoomId,
    /// max number of students
    pub capacity: u32,
    pub travels: Vec<Travel>,
    /// times when the room is unavailable
    pub unavailabilities: Vec<TimeSlots>,
}

impl Travel {
    fn parse(e: &BytesStart) -> Result<Self, ParseError> {
        let mut room = None;
        let mut value = None;

        for attr in e.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let val = std::str::from_utf8(&attr.value)?;

            match key {
                b"room" => room = Some(RoomId(parse_value("room", val)?)),
                b"value" => value = Some(parse_value("value", val)?),
                _ => {
                    return Err(ParseError::UnexpectedAttr(
                        std::str::from_utf8(key)?.to_string(),
                    ));
                }
            }
        }

        Ok(Self {
            room: room.ok_or(ParseError::MissingAttr("room"))?,
            value: value.ok_or(ParseError::MissingAttr("value"))?,
        })
    }
}

impl Room {
    fn parse<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        start: &BytesStart,
        buf: &mut Vec<u8>,
        empty: bool,
    ) -> Result<Self, ParseError> {
        let mut id = None;
        let mut capacity = None;

        for attr in start.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let value = std::str::from_utf8(&attr.value)?;

            match key {
                b"id" => id = Some(RoomId(parse_value("id", value)?)),
                b"capacity" => capacity = Some(parse_value("capacity", value)?),
                _ => {
                    return Err(ParseError::UnexpectedAttr(
                        std::str::from_utf8(key)?.to_string(),
                    ));
                }
            }
        }

        let id = id.ok_or(ParseError::MissingAttr("id"))?;
        let capacity = capacity.ok_or(ParseError::MissingAttr("capacity"))?;

        let mut travels = Vec::new();
        let mut unavailabilities = Vec::new();

        loop {
            if empty {
                break;
            }

            let event = reader.read_event_into(buf)?;
            match event {
                Event::Empty(e) if e.name().as_ref() == b"travel" => {
                    travels.push(Travel::parse(&e)?);
                }
                Event::Empty(e) if e.name().as_ref() == b"unavailable" => {
                    unavailabilities.push(TimeSlots::parse(&e)?);
                }

                Event::End(e) if e.name().as_ref() == b"room" => break,

                _ => {}
            }

            buf.clear();
        }

        Ok(Self {
            id,
            capacity,
            travels,
            unavailabilities,
        })
    }
}

impl Rooms {
    pub(crate) fn parse<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        start: &BytesStart,
        buf: &mut Vec<u8>,
    ) -> Result<Self, ParseError> {
        if start.attributes().next().is_some() {
            return Err(ParseError::UnexpectedAttr("rooms".to_string()));
        }

        let mut rooms = Vec::new();

        loop {
            let event = reader.read_event_into(buf)?;

            match event {
                Event::Start(e) if e.name().as_ref() == b"room" => {
                    let e = e.to_owned();
                    let room = Room::parse(reader, &e, buf, false)?;
                    rooms.push(room);
                }

                Event::Empty(e) if e.name().as_ref() == b"room" => {
                    let e = e.to_owned();
                    let room = Room::parse(reader, &e, buf, true)?;
                    rooms.push(room);
                }

                Event::End(e) if e.name().as_ref() == b"rooms" => break,

                _ => {}
            }

            buf.clear();
        }

        Ok(Self(rooms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{days::Days, timeslots::TimeSlots, utils::prepare, weeks::Weeks};

    #[test]
    fn empty_rooms() {
        let xml = r#"<rooms></rooms>"#;
        let (mut reader, start, mut buf) = prepare(xml);

        let rooms = Rooms::parse(&mut reader, &start, &mut buf).unwrap();

        assert_eq!(rooms, Rooms(vec![]));
    }

    #[test]
    fn multiple_rooms() {
        let xml = r#"
        <rooms>
            <room id="1" capacity="100"/>
            <room id="2" capacity="200"></room>
            <room id="3" capacity="300">
                <unavailable days="1100000" start="102" length="24" weeks="1000000000000"/>
            </room>
        </rooms>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);

        let rooms = Rooms::parse(&mut reader, &start, &mut buf).unwrap();

        assert_eq!(rooms.0.len(), 3);
        assert_eq!(rooms.0[0].id, RoomId(1));
        assert_eq!(rooms.0[0].capacity, 100);
        assert_eq!(rooms.0[1].id, RoomId(2));
        assert_eq!(rooms.0[1].capacity, 200);
        assert_eq!(rooms.0[2].id, RoomId(3));
        assert_eq!(rooms.0[2].capacity, 300);
        assert_eq!(
            rooms.0[2].unavailabilities,
            vec![TimeSlots {
                start: 102,
                length: 24,
                days: Days(1 << 0 | 1 << 1),
                weeks: Weeks(1)
            }]
        )
    }

    #[test]
    fn rooms_unexpected_attribute() {
        let xml = r#"<rooms foo="bar"></rooms>"#;

        let (mut reader, start, mut buf) = prepare(xml);

        let err = Rooms::parse(&mut reader, &start, &mut buf).unwrap_err();

        match err {
            ParseError::UnexpectedAttr(attr) => assert_eq!(attr, "rooms"),
            other => panic!("unexpected error: {:?}", other),
        }
    }
}
