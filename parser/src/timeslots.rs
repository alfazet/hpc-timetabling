use quick_xml::events::BytesStart;

use crate::{ParseError, days::Days, utils::parse_value, weeks::Weeks};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimeSlots {
    /// start time (from the beginning of a day) in time slots
    /// (see [crate::Problem::slots_per_day])
    pub start: u32,
    /// time length in time slots (see [crate::Problem::slots_per_day])
    pub length: u32,
    pub days: Days,
    pub weeks: Weeks,
}

impl TimeSlots {
    pub(crate) fn parse(e: &BytesStart) -> Result<Self, ParseError> {
        let mut start = None;
        let mut length = None;
        let mut days = None;
        let mut weeks = None;

        for attr in e.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let val = std::str::from_utf8(&attr.value)?;

            match key {
                b"start" => start = Some(parse_value("start", val)?),
                b"length" => length = Some(parse_value("length", val)?),
                b"days" => days = Some(Days::parse(val)?),
                b"weeks" => weeks = Some(Weeks::parse(val)?),
                _ => {
                    // allowed to have some other attributes, ex. `penalty`
                }
            }
        }

        Ok(Self {
            start: start.ok_or(ParseError::MissingAttr("start"))?,
            length: length.ok_or(ParseError::MissingAttr("length"))?,
            days: days.ok_or(ParseError::MissingAttr("days"))?,
            weeks: weeks.ok_or(ParseError::MissingAttr("weeks"))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::{Reader, events::Event};

    fn element(xml: &str) -> BytesStart<'static> {
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        match reader.read_event_into(&mut buf).unwrap() {
            Event::Empty(e) | Event::Start(e) => e.to_owned(),
            _ => panic!("expected start element"),
        }
    }

    #[test]
    fn valid_timeslot() {
        let e = element(r#"<time start="90" length="10" days="1010100" weeks="1111111111111"/>"#);

        let ts = TimeSlots::parse(&e).unwrap();

        assert_eq!(ts.start, 90);
        assert_eq!(ts.length, 10);
        assert_eq!(ts.days, Days::parse("1010100").unwrap());
        assert_eq!(ts.weeks, Weeks::parse("1111111111111").unwrap());
    }

    #[test]
    fn valid_timeslot_custom_name() {
        let e =
            element(r#"<helloworld start="90" length="10" days="1010100" weeks="1111111111111"/>"#);

        let ts = TimeSlots::parse(&e).unwrap();

        assert_eq!(ts.start, 90);
        assert_eq!(ts.length, 10);
        assert_eq!(ts.days, Days::parse("1010100").unwrap());
        assert_eq!(ts.weeks, Weeks::parse("1111111111111").unwrap());
    }

    #[test]
    fn fails_on_missing_start() {
        let e = element(r#"<time length="10" days="1010100" weeks="1111111111111"/>"#);

        let err = TimeSlots::parse(&e).unwrap_err();
        assert!(matches!(err, ParseError::MissingAttr("start")));
    }

    #[test]
    fn fails_on_missing_length() {
        let e = element(r#"<time start="90" days="1010100" weeks="1111111111111"/>"#);

        let err = TimeSlots::parse(&e).unwrap_err();
        assert!(matches!(err, ParseError::MissingAttr("length")));
    }

    #[test]
    fn fails_on_missing_days() {
        let e = element(r#"<time start="90" length="10" weeks="1111111111111"/>"#);

        let err = TimeSlots::parse(&e).unwrap_err();
        assert!(matches!(err, ParseError::MissingAttr("days")));
    }

    #[test]
    fn fails_on_missing_weeks() {
        let e = element(r#"<time start="90" length="10" days="1010100"/>"#);

        let err = TimeSlots::parse(&e).unwrap_err();
        assert!(matches!(err, ParseError::MissingAttr("weeks")));
    }

    #[test]
    fn fails_on_invalid_days() {
        let e = element(r#"<time start="90" length="10" days="abc" weeks="1111111111111"/>"#);

        assert!(TimeSlots::parse(&e).is_err());
    }

    #[test]
    fn fails_on_invalid_weeks() {
        let e = element(r#"<time start="90" length="10" days="1010100" weeks="xyz"/>"#);

        assert!(TimeSlots::parse(&e).is_err());
    }

    #[test]
    fn fails_on_invalid_start_number() {
        let e = element(r#"<time start="abc" length="10" days="1010100" weeks="1111111111111"/>"#);

        assert!(TimeSlots::parse(&e).is_err());
    }
}
