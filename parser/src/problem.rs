use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

use crate::{error::ParseError, optimization::Optimization, utils::parse_value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub name: String,
    pub nr_days: u32,
    pub nr_weeks: u32,
    pub slots_per_day: u32,
    pub optimization: Optimization,
}

impl Problem {
    pub fn parse(xml: &str) -> Result<Self, ParseError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        let mut problem_attrs = None;
        let mut optimization = None;

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) if e.name().as_ref() == b"problem" => {
                    problem_attrs = Some(Self::parse_problem_attrs(&e)?);
                }

                Event::Empty(e) if e.name().as_ref() == b"optimization" => {
                    optimization = Some(Optimization::parse(&e)?);
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
    use super::*;

    #[test]
    fn parses_problem_with_optimization() {
        let xml = include_str!("../../data/bet-sum18.xml");

        let problem = Problem::parse(&xml).unwrap();

        assert_eq!(problem.name, "bet-sum18");
        assert_eq!(problem.nr_days, 7);
        assert_eq!(problem.nr_weeks, 6);
        assert_eq!(problem.slots_per_day, 288);
        assert_eq!(
            problem.optimization,
            Optimization {
                time: 1,
                room: 1,
                distribution: 10,
                student: 10,
            }
        )
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
