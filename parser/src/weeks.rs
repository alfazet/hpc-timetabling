use crate::ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Weeks(pub u16);

impl Weeks {
    pub(crate) fn parse(bits: &str) -> Result<Self, ParseError> {
        let mut value = 0u16;

        for (i, c) in bits.chars().enumerate() {
            let max_weeks_per_semester = 15;
            if i >= max_weeks_per_semester {
                return Err(ParseError::InvalidBitString("weeks".into()));
            }

            match c {
                '1' => value |= 1 << i,
                '0' => {}
                _ => return Err(ParseError::InvalidBitString("weeks".into())),
            }
        }

        Ok(Weeks(value))
    }

    pub fn contains(self, week: u8) -> bool {
        (self.0 & (1 << week)) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bitstring() {
        let days = Weeks::parse("1100000").unwrap();

        assert_eq!(days.0, 1 << 0 | 1 << 1);
        for i in 0..2 {
            assert!(days.contains(i));
        }
        for i in 2..7 {
            assert!(!days.contains(i));
        }
    }

    #[test]
    fn parses_all_zero() {
        let days = Weeks::parse("0000000").unwrap();

        for d in 0..7 {
            assert!(!days.contains(d));
        }
    }

    #[test]
    fn parses_all_one() {
        let days = Weeks::parse("1111111").unwrap();

        for d in 0..7 {
            assert!(days.contains(d));
        }
    }

    #[test]
    fn invalid_character() {
        let err = Weeks::parse("10a0000").unwrap_err();

        match err {
            ParseError::InvalidBitString(s) => assert_eq!(s, "weeks"),
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn too_long() {
        let err = Weeks::parse("1110001110001110").unwrap_err();

        match err {
            ParseError::InvalidBitString(s) => assert_eq!(s, "weeks"),
            other => panic!("unexpected error: {:?}", other),
        }
    }
}
