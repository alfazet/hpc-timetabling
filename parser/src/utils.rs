use std::str::FromStr;

use crate::error::ParseError;

pub(crate) fn parse_value<T: FromStr>(attr: &'static str, value: &str) -> Result<T, ParseError> {
    value.parse().map_err(|_| ParseError::InvalidValue {
        attr,
        value: value.to_string(),
    })
}

#[cfg(test)]
use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};
#[cfg(test)]
pub(crate) fn prepare(xml: &str) -> (Reader<&[u8]>, BytesStart<'static>, Vec<u8>) {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    let start = match reader.read_event_into(&mut buf).unwrap() {
        Event::Start(e) => e.to_owned(),
        _ => panic!("expected start element"),
    };

    (reader, start, Vec::new())
}
