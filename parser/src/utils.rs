use std::str::FromStr;

use crate::error::ParseError;

pub(crate) fn parse_value<T: FromStr>(attr: &'static str, value: &str) -> Result<T, ParseError> {
    value.parse().map_err(|_| ParseError::InvalidValue {
        attr,
        value: value.to_string(),
    })
}
