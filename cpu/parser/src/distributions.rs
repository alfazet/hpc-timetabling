//! List of Distribution Constraints: a distribution constraint can be hard
//! (`required="true"`) or soft (has a penalty). For most soft constraints,
//! a penalty is incurred for each pair of classes that violates the constraint.

use std::io::BufRead;

use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

use crate::{ParseError, courses::ClassId, utils::parse_value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Distributions(pub Vec<Distribution>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Distribution {
    pub kind: DistributionKind,
    pub classes: Vec<ClassId>,
    pub penalty: Option<u32>,
}

impl Distribution {
    pub fn required(&self) -> bool {
        self.penalty.is_none()
    }
}

/// the constraint is checked for every pair of classes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistributionKind {
    /// `i.start == j.start`
    SameStart,
    /// `i.start <= j.start && i.end <= j.end` or `j.start <= i.start && j.end <= i.end`
    SameTime,
    /// `i.end <= j.start` or `j.end <= i.start`
    DifferentTime,
    /// `i.days | j.days == i.days` or `i.days | j.days == j.days`
    SameDays,
    /// `i.days & j.days == 0`
    DifferentDays,
    /// `i.weeks | j.weeks == i.weeks` or `i.weeks | j.weeks == j.weeks`
    SameWeeks,
    /// `i.weeks & j.days == 0`
    DifferentWeeks,
    /// `j.start < i.end` and `i.start < j.end` and `i.days & j.days` and `i.weeks & j.weeks`
    Overlap,
    /// opposite of [Self::Overlap]
    NotOverlap,
    /// `i.room == j.room`
    SameRoom,
    /// `i.room != j.room`
    DifferentRoom,
    /// an instructor should be able to attend every class, considering travel
    ///
    /// `i.end + i.travel[j.room] <= j.start` or
    /// `j.end + j.travel[i.room] <= i.start` or
    /// `i.days & j.days == 0` or `(i.weeks & j.weeks == 0)`
    SameAttendees,
    /// considering the ordering in [Distribution::classes], the first lesson
    /// of class `i` must happen before the first lesson of class `j` if `i < j`
    ///
    /// `fst(i.weeks) < fst(j.weeks)` or
    /// (`fst(i.weeks) == fst(j.weeks)` and `fst(i.days) < fst(j.days)` or
    /// `fst(i.days) == fst(j.days)` and `i.end <= j.start`)
    Precedence,
    /// no more than `S` time slots between the start of the first class and the
    /// end of the last class
    ///
    /// `i.weeks & j.weeks == 0` or
    /// `i.days & j.days == 0` or
    /// `max(i.end, j.end) - min(i.start, j.start) <= S`
    WorkDay(u16),
    /// if `i` and `j` happen on same day, there must be a gap of at least `G`
    /// time slots between them
    ///
    /// `i.weeks & j.weeks == 0` or
    /// `i.days & j.days == 0` or
    /// `i.end + G <= j.end` or `j.end + G <= i.end`
    MinGap(u16),
    /// `nonzero_bits(1.days | 2.days | ... n.days) <= D`
    MaxDays(u8),
    /// let `dl(d, w)` be the total amount of time slots assigned to the classes
    /// at day `d` and week `w`
    ///
    /// the constraint requires `dl(d, w) <= S` for all `d` and `w`
    ///
    /// the penalty is multiplied by `sum max(dl(d, w) - S, 0) /` [crate::Problem::nr_weeks]
    MaxDayLoad(u16),
    /// `MaxBreaks(R, S)` - no more than `R` breaks between classes during a day
    /// considering only breaks that are at least `S` time slots long
    MaxBreaks(u16, u16),
    /// `MaxBlock(M, S)` - consecutive classes are considered to be in the same
    /// block if the gap between them `<= S` time slots
    ///
    /// the maximum block size (end of last class `-` start of first class)
    /// should be `<= M`
    ///
    /// only blocks of size >= 2 are considered (a single class `>= M` doesn't
    /// break the constraint)
    ///
    /// the penalty is multiplied by the total number of blocks `> M` over all
    /// days and all weeks divided by [crate::Problem::nr_weeks]
    MaxBlock(u16, u16),
}

impl DistributionKind {
    fn parse(s: &str) -> Result<DistributionKind, ParseError> {
        macro_rules! parse_param {
            ($s:expr, $variant:ident) => {{
                let name = stringify!($variant);
                let Some(arg) = $s
                    .strip_prefix(name)
                    .and_then(|v| v.strip_prefix('('))
                    .and_then(|v| v.strip_suffix(')'))
                else {
                    return Err(ParseError::InvalidValue {
                        attr: name.into(),
                        value: $s.into(),
                    });
                };
                let val = parse_value(name, arg)?;
                Ok(DistributionKind::$variant(val))
            }};
            ($s:expr, $variant:ident, two) => {{
                let name = stringify!($variant);
                let invalid_value = || ParseError::InvalidValue {
                    attr: name.into(),
                    value: $s.into(),
                };
                let Some(arg) = $s
                    .strip_prefix(name)
                    .and_then(|v| v.strip_prefix('('))
                    .and_then(|v| v.strip_suffix(')'))
                else {
                    return Err(invalid_value());
                };

                let mut parts = arg.split(',');
                let first = parts.next().ok_or_else(invalid_value)?;
                let first = parse_value(name, first)?;
                let second = parts.next().ok_or_else(invalid_value)?;
                let second = parse_value(name, second)?;
                if parts.next().is_some() {
                    return Err(invalid_value());
                }
                Ok(DistributionKind::$variant(first, second))
            }};
        }

        match s {
            "SameStart" => return Ok(DistributionKind::SameStart),
            "SameTime" => return Ok(DistributionKind::SameTime),
            "DifferentTime" => return Ok(DistributionKind::DifferentTime),
            "SameDays" => return Ok(DistributionKind::SameDays),
            "DifferentDays" => return Ok(DistributionKind::DifferentDays),
            "SameWeeks" => return Ok(DistributionKind::SameWeeks),
            "DifferentWeeks" => return Ok(DistributionKind::DifferentWeeks),
            "Overlap" => return Ok(DistributionKind::Overlap),
            "NotOverlap" => return Ok(DistributionKind::NotOverlap),
            "SameRoom" => return Ok(DistributionKind::SameRoom),
            "DifferentRoom" => return Ok(DistributionKind::DifferentRoom),
            "SameAttendees" => return Ok(DistributionKind::SameAttendees),
            "Precedence" => return Ok(DistributionKind::Precedence),
            _ => {
                // parameterized types, parsed below
            }
        }

        if s.starts_with("WorkDay") {
            return parse_param!(s, WorkDay);
        }
        if s.starts_with("MinGap") {
            return parse_param!(s, MinGap);
        }
        if s.starts_with("MaxDays") {
            return parse_param!(s, MaxDays);
        }
        if s.starts_with("MaxDayLoad") {
            return parse_param!(s, MaxDayLoad);
        }
        if s.starts_with("MaxBreaks") {
            return parse_param!(s, MaxBreaks, two);
        }
        if s.starts_with("MaxBlock") {
            return parse_param!(s, MaxBlock, two);
        }

        Err(ParseError::InvalidValue {
            attr: "type",
            value: s.into(),
        })
    }
}

impl Distribution {
    fn parse<R: BufRead>(
        reader: &mut Reader<R>,
        start: &BytesStart,
        buf: &mut Vec<u8>,
    ) -> Result<Self, ParseError> {
        let mut kind = None;
        let mut penalty = None;

        for attr in start.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let val = std::str::from_utf8(&attr.value)?;

            match key {
                b"type" => kind = Some(DistributionKind::parse(val)?),
                b"penalty" => penalty = Some(parse_value("penalty", val)?),
                b"required" => {
                    // handled by `penalty.is_none()`
                }
                _ => {
                    return Err(ParseError::UnexpectedAttr(
                        std::str::from_utf8(key)?.to_string(),
                    ));
                }
            }
        }

        let kind = kind.ok_or(ParseError::MissingAttr("type"))?;

        let mut classes = Vec::new();

        loop {
            match reader.read_event_into(buf)? {
                Event::Empty(e) if e.name().as_ref() == b"class" => {
                    classes.push(Self::parse_class(&e)?);
                }

                Event::End(e) if e.name().as_ref() == b"distribution" => break,

                _ => {}
            }

            buf.clear();
        }

        Ok(Self {
            kind,
            classes,
            penalty,
        })
    }

    fn parse_class(e: &BytesStart) -> Result<ClassId, ParseError> {
        let mut id = None;

        for attr in e.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let val = std::str::from_utf8(&attr.value)?;

            match key {
                b"id" => id = Some(ClassId::new(parse_value("id", val)?)),
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

impl Distributions {
    pub(crate) fn parse<R: BufRead>(
        reader: &mut Reader<R>,
        start: &BytesStart,
        buf: &mut Vec<u8>,
    ) -> Result<Self, ParseError> {
        if start.attributes().next().is_some() {
            return Err(ParseError::UnexpectedAttr("distributions".into()));
        }

        let mut distributions = Vec::new();

        loop {
            match reader.read_event_into(buf)? {
                Event::Start(e) if e.name().as_ref() == b"distribution" => {
                    let e = e.to_owned();
                    buf.clear();

                    let distribution = Distribution::parse(reader, &e, buf)?;
                    distributions.push(distribution);
                }

                Event::End(e) if e.name().as_ref() == b"distributions" => break,

                _ => {}
            }

            buf.clear();
        }

        Ok(Self(distributions))
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::prepare;

    use super::*;

    #[test]
    fn simple_kinds() {
        assert_eq!(
            DistributionKind::parse("SameStart").unwrap(),
            DistributionKind::SameStart
        );
        assert_eq!(
            DistributionKind::parse("SameTime").unwrap(),
            DistributionKind::SameTime
        );
        assert_eq!(
            DistributionKind::parse("DifferentTime").unwrap(),
            DistributionKind::DifferentTime
        );
        assert_eq!(
            DistributionKind::parse("NotOverlap").unwrap(),
            DistributionKind::NotOverlap
        );
        assert_eq!(
            DistributionKind::parse("SameAttendees").unwrap(),
            DistributionKind::SameAttendees
        );
        assert_eq!(
            DistributionKind::parse("Precedence").unwrap(),
            DistributionKind::Precedence
        );
    }

    #[test]
    fn single_parameter_kinds() {
        assert_eq!(
            DistributionKind::parse("WorkDay(5)").unwrap(),
            DistributionKind::WorkDay(5)
        );

        assert_eq!(
            DistributionKind::parse("MinGap(3)").unwrap(),
            DistributionKind::MinGap(3)
        );

        assert_eq!(
            DistributionKind::parse("MaxDays(2)").unwrap(),
            DistributionKind::MaxDays(2)
        );

        assert_eq!(
            DistributionKind::parse("MaxDayLoad(100)").unwrap(),
            DistributionKind::MaxDayLoad(100)
        );
    }

    #[test]
    fn two_parameter_kinds() {
        assert_eq!(
            DistributionKind::parse("MaxBreaks(2,10)").unwrap(),
            DistributionKind::MaxBreaks(2, 10)
        );

        assert_eq!(
            DistributionKind::parse("MaxBlock(3,20)").unwrap(),
            DistributionKind::MaxBlock(3, 20)
        );
    }

    #[test]
    fn invalid_parameter_values() {
        assert!(DistributionKind::parse("MaxDays(x)").is_err());
        assert!(DistributionKind::parse("MaxBreaks(1,x)").is_err());
    }

    #[test]
    fn invalid_parameter_count() {
        assert!(DistributionKind::parse("MaxBreaks(1)").is_err());
        assert!(DistributionKind::parse("MaxBreaks(1,2,3)").is_err());
    }

    #[test]
    fn invalid_type_name() {
        assert!(DistributionKind::parse("UnknownConstraint").is_err());
    }

    #[test]
    fn simple_distribution() {
        let xml = r#"
        <distribution type="NotOverlap" penalty="5">
            <class id="1"/>
            <class id="2"/>
        </distribution>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);

        let dist = Distribution::parse(&mut reader, &start, &mut buf).unwrap();

        assert_eq!(dist.kind, DistributionKind::NotOverlap);
        assert_eq!(dist.penalty, Some(5));
        assert_eq!(dist.classes.len(), 2);
        assert_eq!(dist.classes[0], ClassId::new(1));
        assert_eq!(dist.classes[1], ClassId::new(2));
    }

    #[test]
    fn parses_parameterized_distribution() {
        let xml = r#"
        <distribution type="MaxDays(2)">
            <class id="3"/>
            <class id="5"/>
        </distribution>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);

        let dist = Distribution::parse(&mut reader, &start, &mut buf).unwrap();

        assert_eq!(dist.kind, DistributionKind::MaxDays(2));
        assert_eq!(dist.penalty, None);
        assert_eq!(dist.classes, vec![ClassId::new(3), ClassId::new(5)]);
    }

    #[test]
    fn distribution_missing_type() {
        let xml = r#"
        <distribution penalty="3">
            <class id="1"/>
        </distribution>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);

        assert!(Distribution::parse(&mut reader, &start, &mut buf).is_err());
    }

    #[test]
    fn class_unexpected_attribute() {
        let xml = r#"
        <distribution type="NotOverlap">
            <class id="1" foo="bar"/>
        </distribution>
        "#;

        let (mut reader, start, mut buf) = prepare(xml);

        assert!(Distribution::parse(&mut reader, &start, &mut buf).is_err());
    }
}
