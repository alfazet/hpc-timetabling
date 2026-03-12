use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

use crate::{error::ParseError, optimization::Optimization, rooms::Rooms, utils::parse_value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub name: String,
    /// number of days in a week
    pub nr_days: u32,
    /// number of weeks in a semester
    pub nr_weeks: u32,
    /// number of time slots in a day
    /// (usually `288 = 24 * 60 / 5`, meaning 288 5 min slots in 24 h)
    pub slots_per_day: u32,
    pub optimization: Optimization,
    pub rooms: Rooms,
}

impl Problem {
    pub fn parse(xml: &str) -> Result<Self, ParseError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        let mut problem_attrs = None;
        let mut optimization = None;
        let mut rooms = None;

        loop {
            let event = reader.read_event_into(&mut buf)?;
            match event {
                Event::Start(e) if e.name().as_ref() == b"problem" => {
                    problem_attrs = Some(Self::parse_problem_attrs(&e)?);
                }

                Event::Empty(e) if e.name().as_ref() == b"optimization" => {
                    optimization = Some(Optimization::parse(&e)?);
                }

                Event::Start(e) if e.name().as_ref() == b"rooms" => {
                    let e = e.to_owned();
                    rooms = Some(Rooms::parse(&mut reader, &e, &mut buf)?);
                }

                Event::Eof => break,

                _ => {}
            }

            buf.clear();
        }

        let (name, nr_days, nr_weeks, slots_per_day) =
            problem_attrs.ok_or(ParseError::MissingElement("problem"))?;

        Ok(Problem {
            name,
            nr_days,
            nr_weeks,
            slots_per_day,
            optimization: optimization.ok_or(ParseError::MissingElement("optimization"))?,
            rooms: rooms.ok_or(ParseError::MissingElement("rooms"))?,
        })
    }

    fn parse_problem_attrs(e: &BytesStart) -> Result<(String, u32, u32, u32), ParseError> {
        let mut name = None;
        let mut nr_days = None;
        let mut nr_weeks = None;
        let mut slots_per_day = None;

        for attr in e.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let value = std::str::from_utf8(&attr.value)?;

            match key {
                b"name" => name = Some(value.to_string()),
                b"nrDays" => nr_days = Some(parse_value("nrDays", value)?),
                b"nrWeeks" => nr_weeks = Some(parse_value("nrWeeks", value)?),
                b"slotsPerDay" => slots_per_day = Some(parse_value("slotsPerDay", value)?),
                _ => {
                    return Err(ParseError::UnexpectedAttr(
                        std::str::from_utf8(key)?.to_string(),
                    ));
                }
            }
        }

        Ok((
            name.ok_or(ParseError::MissingAttr("name"))?,
            nr_days.ok_or(ParseError::MissingAttr("nrDays"))?,
            nr_weeks.ok_or(ParseError::MissingAttr("nrWeeks"))?,
            slots_per_day.ok_or(ParseError::MissingAttr("slotsPerDay"))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        days::Days,
        rooms::{Room, RoomId, Travel, Unavailable},
        weeks::Weeks,
    };

    use super::*;

    #[test]
    fn parses_problem_with_optimization() {
        let xml = include_str!("../../data/itc2019/sample.xml");

        let problem = Problem::parse(&xml).unwrap();

        dbg!(&problem);
        assert_eq!(
            problem,
            Problem {
                name: "unique-instance-name".into(),
                nr_days: 7,
                nr_weeks: 13,
                slots_per_day: 288,
                optimization: Optimization {
                    time: 2,
                    room: 1,
                    distribution: 1,
                    student: 2
                },
                rooms: Rooms(vec![
                    Room {
                        id: RoomId(1),
                        capacity: 50,
                        travels: vec![],
                        unavailabilities: vec![],
                    },
                    Room {
                        id: RoomId(2),
                        capacity: 100,
                        travels: vec![Travel {
                            room: RoomId(1),
                            value: 2,
                        }],
                        unavailabilities: vec![],
                    },
                    Room {
                        id: RoomId(3),
                        capacity: 80,
                        travels: vec![Travel {
                            room: RoomId(2),
                            value: 3,
                        }],
                        unavailabilities: vec![
                            Unavailable {
                                start: 102,
                                length: 24,
                                days: Days(3),
                                weeks: Weeks(u16::from_str_radix("1111111111111", 2).unwrap()),
                            },
                            Unavailable {
                                start: 144,
                                length: 144,
                                days: Days(8),
                                weeks: Weeks(u16::from_str_radix("1010101010101", 2).unwrap()),
                            }
                        ],
                    },
                ]),
            }
        );
    }

    #[test]
    fn parses_all_instances() {
        fn visit_dir(dir: &std::path::Path) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();

                if path.is_dir() {
                    visit_dir(&path);
                } else if path.extension().is_some_and(|e| e == "xml") {
                    let xml = std::fs::read_to_string(&path).unwrap();
                    Problem::parse(&xml)
                        .unwrap_or_else(|e| panic!("failed to parse {:?}: {e}", path));
                }
            }
        }

        visit_dir(std::path::Path::new("../data"));
    }
}
