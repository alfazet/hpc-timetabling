//! This is the most complex part.
//!
//! 1) student chooses some courses
//! 2) algorithm is free to choose on of each the courses' configs PER STUDENT
//! 3) within each subpart of the choosen config, the student is assigned to
//!    exactly one class (not already full)
//! 4) within each class the algorithm must choose exactly one time and one room,
//!    except it is possible for a class to have no rooms

use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

use crate::{
    ParseError,
    rooms::RoomId,
    timeslots::TimeSlots,
    utils::{define_id, parse_value},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Courses(pub Vec<Course>);

define_id!(CourseId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Course {
    pub id: CourseId,
    pub configs: Vec<Config>,
}

define_id!(ConfigId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub id: ConfigId,
    pub subparts: Vec<Subpart>,
}

define_id!(SubpartId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subpart {
    pub id: SubpartId,
    pub classes: Vec<Class>,
}

define_id!(ClassId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    pub id: ClassId,
    /// max number of students
    pub limit: Option<u32>,
    /// student taking this class must also take its parent
    pub parent: Option<ClassId>,
    /// can be empty, otherwise the algorithm has to choose exactly one
    pub rooms: Vec<ClassRoom>,
    /// nonempty, the algorithm has to choose exactly one
    pub times: Vec<ClassTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassRoom {
    pub room: RoomId,
    /// score penalty for picking this room for the class
    pub penalty: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassTime {
    pub times: TimeSlots,
    /// score penalty for picking this time slot for the class
    pub penalty: u32,
}

impl ClassRoom {
    fn parse(e: &BytesStart) -> Result<Self, ParseError> {
        let mut room = None;
        let mut penalty = None;

        for attr in e.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let val = std::str::from_utf8(&attr.value)?;

            match key {
                b"id" => room = Some(RoomId::new(parse_value("id", val)?)),
                b"penalty" => penalty = Some(parse_value("penalty", val)?),
                _ => {
                    return Err(ParseError::UnexpectedAttr(
                        std::str::from_utf8(key)?.to_string(),
                    ));
                }
            }
        }

        Ok(Self {
            room: room.ok_or(ParseError::MissingAttr("id"))?,
            penalty: penalty.ok_or(ParseError::MissingAttr("penalty"))?,
        })
    }
}

impl ClassTime {
    fn parse(e: &BytesStart) -> Result<Self, ParseError> {
        let mut penalty = None;

        for attr in e.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let val = std::str::from_utf8(&attr.value)?;

            match key {
                b"penalty" => penalty = Some(parse_value("penalty", val)?),
                _ => {
                    // other keys parsed by [`TimeSlots::parse`]
                }
            }
        }

        let times = TimeSlots::parse(e)?;

        Ok(Self {
            times,
            penalty: penalty.ok_or(ParseError::MissingAttr("penalty"))?,
        })
    }
}

impl Class {
    fn parse<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        start: &BytesStart,
        buf: &mut Vec<u8>,
    ) -> Result<Self, ParseError> {
        let mut id = None;
        let mut limit = None;
        let mut parent = None;

        for attr in start.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let val = std::str::from_utf8(&attr.value)?;

            match key {
                b"id" => id = Some(ClassId(parse_value("id", val)?)),
                b"limit" => limit = Some(parse_value("limit", val)?),
                b"parent" => parent = Some(ClassId(parse_value("parent", val)?)),
                b"room" => {
                    // may have `room="false"` but that's just represented by
                    // having empty class.rooms
                }
                _ => {
                    return Err(ParseError::UnexpectedAttr(
                        std::str::from_utf8(key)?.to_string(),
                    ));
                }
            }
        }

        let id = id.ok_or(ParseError::MissingAttr("id"))?;

        let mut rooms = Vec::new();
        let mut times = Vec::new();

        loop {
            let event = reader.read_event_into(buf)?;

            match event {
                Event::Empty(e) if e.name().as_ref() == b"room" => {
                    let e = e.to_owned();
                    rooms.push(ClassRoom::parse(&e)?);
                }

                Event::Empty(e) if e.name().as_ref() == b"time" => {
                    let e = e.to_owned();
                    times.push(ClassTime::parse(&e)?);
                }

                Event::End(e) if e.name().as_ref() == b"class" => break,

                _ => {}
            }

            buf.clear();
        }

        if times.is_empty() {
            return Err(ParseError::MissingElement("time"));
        }

        Ok(Self {
            id,
            limit,
            parent,
            rooms,
            times,
        })
    }
}

impl Subpart {
    fn parse<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        start: &BytesStart,
        buf: &mut Vec<u8>,
    ) -> Result<Self, ParseError> {
        let mut id = None;

        for attr in start.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let val = std::str::from_utf8(&attr.value)?;

            match key {
                b"id" => id = Some(SubpartId::new(parse_value("id", val)?)),
                _ => {
                    return Err(ParseError::UnexpectedAttr(
                        std::str::from_utf8(key)?.to_string(),
                    ));
                }
            }
        }

        let id = id.ok_or(ParseError::MissingAttr("id"))?;

        let mut classes = Vec::new();

        loop {
            let event = reader.read_event_into(buf)?;

            match event {
                Event::Start(e) if e.name().as_ref() == b"class" => {
                    let e = e.to_owned();
                    buf.clear();
                    classes.push(Class::parse(reader, &e, buf)?);
                }

                Event::End(e) if e.name().as_ref() == b"subpart" => break,

                _ => {}
            }

            buf.clear();
        }

        Ok(Self { id, classes })
    }
}

impl Config {
    fn parse<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        start: &BytesStart,
        buf: &mut Vec<u8>,
    ) -> Result<Self, ParseError> {
        let mut id = None;

        for attr in start.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let val = std::str::from_utf8(&attr.value)?;

            match key {
                b"id" => id = Some(ConfigId::new(parse_value("id", val)?)),
                _ => {
                    return Err(ParseError::UnexpectedAttr(
                        std::str::from_utf8(key)?.to_string(),
                    ));
                }
            }
        }

        let id = id.ok_or(ParseError::MissingAttr("id"))?;

        let mut subparts = Vec::new();

        loop {
            let event = reader.read_event_into(buf)?;

            match event {
                Event::Start(e) if e.name().as_ref() == b"subpart" => {
                    let e = e.to_owned();
                    buf.clear();
                    subparts.push(Subpart::parse(reader, &e, buf)?);
                }

                Event::End(e) if e.name().as_ref() == b"config" => break,

                _ => {}
            }

            buf.clear();
        }

        Ok(Self { id, subparts })
    }
}

impl Course {
    fn parse<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        start: &BytesStart,
        buf: &mut Vec<u8>,
    ) -> Result<Self, ParseError> {
        let mut id = None;

        for attr in start.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let val = std::str::from_utf8(&attr.value)?;

            match key {
                b"id" => id = Some(CourseId(parse_value("id", val)?)),
                _ => {
                    return Err(ParseError::UnexpectedAttr(
                        std::str::from_utf8(key)?.to_string(),
                    ));
                }
            }
        }

        let id = id.ok_or(ParseError::MissingAttr("id"))?;

        let mut configs = Vec::new();

        loop {
            let event = reader.read_event_into(buf)?;

            match event {
                Event::Start(e) if e.name().as_ref() == b"config" => {
                    let e = e.to_owned();
                    buf.clear();
                    configs.push(Config::parse(reader, &e, buf)?);
                }

                Event::End(e) if e.name().as_ref() == b"course" => break,

                _ => {}
            }

            buf.clear();
        }

        Ok(Self { id, configs })
    }
}

impl Courses {
    pub(crate) fn parse<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        start: &BytesStart,
        buf: &mut Vec<u8>,
    ) -> Result<Self, ParseError> {
        if start.attributes().next().is_some() {
            return Err(ParseError::UnexpectedAttr("courses".to_string()));
        }

        let mut courses = Vec::new();

        loop {
            let event = reader.read_event_into(buf)?;

            match event {
                Event::Start(e) if e.name().as_ref() == b"course" => {
                    let e = e.to_owned();
                    buf.clear();
                    courses.push(Course::parse(reader, &e, buf)?);
                }

                Event::End(e) if e.name().as_ref() == b"courses" => break,

                _ => {}
            }

            buf.clear();
        }

        Ok(Self(courses))
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::prepare;

    use super::*;

    #[test]
    fn single_course_structure() {
        let xml = r#"
        <courses>
            <course id="1">
                <config id="1">
                    <subpart id="1">
                        <class id="1" limit="20">
                            <time days="1000000" start="10" length="5" weeks="1111111111111" penalty="0"/>
                        </class>
                    </subpart>
                </config>
            </course>
        </courses>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);

        let courses = Courses::parse(&mut reader, &start, &mut buf).unwrap();

        assert_eq!(courses.0.len(), 1);

        let course = &courses.0[0];
        assert_eq!(course.id, CourseId(1));
        assert_eq!(course.configs.len(), 1);

        let config = &course.configs[0];
        assert_eq!(config.id, ConfigId(1));
        assert_eq!(config.subparts.len(), 1);

        let subpart = &config.subparts[0];
        assert_eq!(subpart.id, SubpartId(1));
        assert_eq!(subpart.classes.len(), 1);

        let class = &subpart.classes[0];
        assert_eq!(class.id, ClassId(1));
        assert_eq!(class.limit, Some(20));
        assert_eq!(class.times.len(), 1);
    }

    #[test]
    fn class_with_room() {
        let xml = r#"
        <courses>
            <course id="1">
                <config id="1">
                    <subpart id="1">
                        <class id="1" limit="10">
                            <room id="5" penalty="2"/>
                            <time days="1000000" start="10" length="5" weeks="1111111111111" penalty="0"/>
                        </class>
                    </subpart>
                </config>
            </course>
        </courses>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);
        let courses = Courses::parse(&mut reader, &start, &mut buf).unwrap();

        let class = &courses.0[0].configs[0].subparts[0].classes[0];

        assert_eq!(class.rooms.len(), 1);
        assert_eq!(class.rooms[0].room, RoomId::new(5));
        assert_eq!(class.rooms[0].penalty, 2);
    }

    #[test]
    fn class_with_parent() {
        let xml = r#"
        <courses>
            <course id="1">
                <config id="1">
                    <subpart id="1">
                        <class id="1" limit="20">
                            <time days="1000000" start="10" length="5" weeks="1111111111111" penalty="0"/>
                        </class>
                        <class id="2" limit="20" parent="1">
                            <time days="0100000" start="20" length="5" weeks="1111111111111" penalty="0"/>
                        </class>
                    </subpart>
                </config>
            </course>
        </courses>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);
        let courses = Courses::parse(&mut reader, &start, &mut buf).unwrap();

        let classes = &courses.0[0].configs[0].subparts[0].classes;

        assert_eq!(classes[1].parent, Some(ClassId(1)));
    }

    #[test]
    fn class_must_have_time() {
        let xml = r#"
        <courses>
            <course id="1">
                <config id="1">
                    <subpart id="1">
                        <class id="1" limit="20">
                        </class>
                    </subpart>
                </config>
            </course>
        </courses>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);

        let result = Courses::parse(&mut reader, &start, &mut buf);

        assert!(result.is_err());
    }

    #[test]
    fn multiple_courses() {
        let xml = r#"
        <courses>
            <course id="1">
                <config id="1">
                    <subpart id="1">
                        <class id="1" limit="10">
                            <time days="1000000" start="10" length="5" weeks="1111111111111" penalty="0"/>
                        </class>
                    </subpart>
                </config>
            </course>
            <course id="2">
                <config id="2">
                    <subpart id="3">
                        <class id="5" limit="15">
                            <time days="0100000" start="20" length="6" weeks="1111111111111" penalty="1"/>
                        </class>
                    </subpart>
                </config>
            </course>
        </courses>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);

        let courses = Courses::parse(&mut reader, &start, &mut buf).unwrap();

        assert_eq!(courses.0.len(), 2);
        assert_eq!(courses.0[0].id, CourseId(1));
        assert_eq!(courses.0[1].id, CourseId(2));
    }
}
