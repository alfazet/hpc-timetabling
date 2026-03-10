use crate::ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Days(pub u8);

impl Days {
    pub(crate) fn parse(bits: &str) -> Result<Self, ParseError> {
        let mut value = 0u8;

        for (i, c) in bits.chars().enumerate() {
            let max_days_per_week = 7;
            if i >= max_days_per_week {
                return Err(ParseError::InvalidBitString("days".into()));
            }

            match c {
                '1' => value |= 1 << i,
                '0' => {}
                _ => return Err(ParseError::InvalidBitString("days".into())),
            }
        }

        Ok(Days(value))
    }

    pub fn contains(self, day: u8) -> bool {
        (self.0 & (1 << day)) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bitstring() {
        let days = Days::parse("1100000").unwrap();

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
        let days = Days::parse("0000000").unwrap();

        for d in 0..7 {
            assert!(!days.contains(d));
        }
    }

    #[test]
    fn parses_all_one() {
        let days = Days::parse("1111111").unwrap();

        for d in 0..7 {
            assert!(days.contains(d));
        }
    }

    #[test]
    fn invalid_character() {
        let err = Days::parse("10a0000").unwrap_err();

        match err {
            ParseError::InvalidBitString(s) => assert_eq!(s, "days"),
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn too_long() {
        let err = Days::parse("10000000").unwrap_err();

        match err {
            ParseError::InvalidBitString(s) => assert_eq!(s, "days"),
            other => panic!("unexpected error: {:?}", other),
        }
    }
}
