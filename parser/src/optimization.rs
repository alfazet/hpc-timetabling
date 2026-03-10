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

        Ok(Optimization {
            time: time.ok_or(ParseError::MissingAttr("time"))?,
            room: room.ok_or(ParseError::MissingAttr("room"))?,
            distribution: distribution.ok_or(ParseError::MissingAttr("distribution"))?,
            student: student.ok_or(ParseError::MissingAttr("student"))?,
        })
    }
}
