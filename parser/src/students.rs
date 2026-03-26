//! Student Course Demands: Each student needs a class of each subpart of one
//! configuration of a course.

use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

use crate::{
    ParseError,
    courses::CourseId,
    utils::{define_id, parse_value},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Students(pub Vec<Student>);

define_id!(StudentId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Student {
    pub id: StudentId,
    pub courses: Vec<CourseId>,
}

impl Student {
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
                b"id" => id = Some(StudentId(parse_value("id", val)?)),
                _ => {
                    return Err(ParseError::UnexpectedAttr(
                        std::str::from_utf8(key)?.to_string(),
                    ));
                }
            }
        }

        let id = id.ok_or(ParseError::MissingAttr("id"))?;

        let mut courses = Vec::new();

        loop {
            let event = reader.read_event_into(buf)?;

            match event {
                Event::Empty(e) if e.name().as_ref() == b"course" => {
                    let e = e.to_owned();
                    courses.push(Self::parse_course(&e)?);
                }

                Event::End(e) if e.name().as_ref() == b"student" => break,

                _ => {}
            }

            buf.clear();
        }

        Ok(Self { id, courses })
    }

    fn parse_course(e: &BytesStart) -> Result<CourseId, ParseError> {
        let mut id = None;

        for attr in e.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let val = std::str::from_utf8(&attr.value)?;

            match key {
                b"id" => id = Some(CourseId::new(parse_value("id", val)?)),
                _ => {
                    return Err(ParseError::UnexpectedAttr(
                        std::str::from_utf8(key)?.to_string(),
                    ));
                }
            }
        }

        id.ok_or(ParseError::MissingAttr("id"))
    }
}

impl Students {
    pub(crate) fn parse<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        start: &BytesStart,
        buf: &mut Vec<u8>,
    ) -> Result<Self, ParseError> {
        if start.attributes().next().is_some() {
            return Err(ParseError::UnexpectedAttr("students".to_string()));
        }

        let mut students = Vec::new();

        loop {
            let event = reader.read_event_into(buf)?;

            match event {
                Event::Start(e) if e.name().as_ref() == b"student" => {
                    let e = e.to_owned();
                    buf.clear();
                    students.push(Student::parse(reader, &e, buf)?);
                }

                Event::End(e) if e.name().as_ref() == b"students" => break,

                _ => {}
            }

            buf.clear();
        }

        Ok(Self(students))
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::prepare;

    use super::*;

    #[test]
    fn students() {
        let xml = r#"
        <students>
            <student id="1">
                <course id="1"/>
                <course id="5"/>
            </student>
            <student id="2">
                <course id="1"/>
                <course id="3"/>
                <course id="4"/>
            </student>
        </students>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);

        let students = Students::parse(&mut reader, &start, &mut buf).unwrap();

        assert_eq!(students.0.len(), 2);

        assert_eq!(students.0[0].id, StudentId(1));
        assert_eq!(
            students.0[0].courses,
            vec![CourseId::new(1), CourseId::new(5)]
        );

        assert_eq!(students.0[1].id, StudentId(2));
        assert_eq!(
            students.0[1].courses,
            vec![CourseId::new(1), CourseId::new(3), CourseId::new(4)]
        );
    }

    #[test]
    fn empty_students() {
        let xml = r#"
        <students></students>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);

        let students = Students::parse(&mut reader, &start, &mut buf).unwrap();

        assert_eq!(students, Students(vec![]));
    }

    #[test]
    fn student_with_no_courses() {
        let xml = r#"
        <students>
            <student id="1"></student>
        </students>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);

        let students = Students::parse(&mut reader, &start, &mut buf).unwrap();

        assert_eq!(students.0.len(), 1);
        assert!(students.0[0].courses.is_empty());
    }

    #[test]
    fn student_missing_id_fails() {
        let xml = r#"
        <students>
            <student>
                <course id="1"/>
            </student>
        </students>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);

        matches!(
            Students::parse(&mut reader, &start, &mut buf),
            Err(ParseError::MissingAttr("id"))
        );
    }

    #[test]
    fn course_missing_id_fails() {
        let xml = r#"
        <students>
            <student id="1">
                <course/>
            </student>
        </students>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);

        matches!(
            Students::parse(&mut reader, &start, &mut buf),
            Err(ParseError::MissingAttr("id"))
        );
    }
}
