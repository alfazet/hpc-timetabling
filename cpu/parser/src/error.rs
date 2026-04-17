#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("XML error")]
    Xml(#[from] quick_xml::Error),

    #[error("XML attribute error")]
    Attr(#[from] quick_xml::events::attributes::AttrError),

    #[error("UTF8 error")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Missing attribute `{0}`")]
    MissingAttr(&'static str),

    #[error("Missing element `{0}`")]
    MissingElement(&'static str),

    #[error("Unexpected attribute `{0}`")]
    UnexpectedAttr(String),

    #[error("Invalid value for `{attr}`: `{value}`")]
    InvalidValue { attr: &'static str, value: String },

    #[error("Invalid bit string for `{0}`")]
    InvalidBitString(String),
}
