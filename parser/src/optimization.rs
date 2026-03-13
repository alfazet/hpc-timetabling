use quick_xml::events::BytesStart;

use crate::{error::ParseError, utils::parse_value};

/// optimization weights: weights on the total penalty of the solution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Optimization {
    /// penalty for assigned times
    pub time: i32,
    /// penalty for assigned rooms
    pub room: i32,
    /// penalty for violated soft distribution constraints
    pub distribution: i32,
    /// penalty for student conflicts
    pub student: i32,
}

impl Optimization {
    pub(crate) fn parse(e: &BytesStart) -> Result<Self, ParseError> {
        let mut time = None;
        let mut room = None;
        let mut distribution = None;
        let mut student = None;

        for attr in e.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let value = std::str::from_utf8(&attr.value)?;

            match key {
                b"time" => time = Some(parse_value("time", value)?),
                b"room" => room = Some(parse_value("room", value)?),
                b"distribution" => distribution = Some(parse_value("distribution", value)?),
                b"student" => student = Some(parse_value("student", value)?),
                _ => {
                    return Err(ParseError::UnexpectedAttr(
                        std::str::from_utf8(key)?.to_string(),
                    ));
                }
            }
        }

        Ok(Self {
            time: time.ok_or(ParseError::MissingAttr("time"))?,
            room: room.ok_or(ParseError::MissingAttr("room"))?,
            distribution: distribution.ok_or(ParseError::MissingAttr("distribution"))?,
            student: student.ok_or(ParseError::MissingAttr("student"))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::{Reader, events::Event};

    fn start_event(xml: &str) -> BytesStart<'_> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        match reader.read_event_into(&mut buf).unwrap() {
            Event::Empty(e) => e.to_owned(),
            Event::Start(e) => e.to_owned(),
            other => panic!("expected start or empty event, got {:?}", other),
        }
    }

    #[test]
    fn parses_valid_optimization() {
        let e = start_event(r#"<optimization time="1" room="2" distribution="3" student="4"/>"#);

        let opt = Optimization::parse(&e).unwrap();

        assert_eq!(
            opt,
            Optimization {
                time: 1,
                room: 2,
                distribution: 3,
                student: 4,
            }
        );
    }

    #[test]
    fn missing_time_attr() {
        let e = start_event(r#"<optimization room="2" distribution="3" student="4"/>"#);

        let err = Optimization::parse(&e).unwrap_err();
        assert!(matches!(err, ParseError::MissingAttr("time")));
    }

    #[test]
    fn unexpected_attribute() {
        let e = start_event(
            r#"<optimization time="1" room="2" distribution="3" student="4" foo="5"/>"#,
        );

        let err = Optimization::parse(&e).unwrap_err();

        match err {
            ParseError::UnexpectedAttr(attr) => assert_eq!(attr, "foo"),
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn invalid_integer_value() {
        let e = start_event(r#"<optimization time="x" room="2" distribution="3" student="4"/>"#);

        let err = Optimization::parse(&e).unwrap_err();

        match err {
            ParseError::InvalidValue {
                attr: "time",
                value,
            } => assert_eq!(value, "x"),
            other => panic!("unexpected error: {:?}", other),
        }
    }
}
