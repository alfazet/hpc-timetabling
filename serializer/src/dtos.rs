use serde::Serialize;

use crate::{
    output::{Class, OutputMetadata},
    utils::{bit_string_u8, bit_string_u16},
};

#[derive(Serialize)]
#[serde(rename = "solution")]
pub(crate) struct XmlOutput {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@runtime")]
    pub runtime: f32,
    #[serde(rename = "@cores")]
    pub cores: usize,
    #[serde(rename = "@technique")]
    pub technique: String,
    #[serde(rename = "@author")]
    pub author: String,
    #[serde(rename = "@institution")]
    pub institution: String,
    #[serde(rename = "@country")]
    pub country: String,
    #[serde(rename = "class")]
    pub classes: Vec<XmlClass>,
}

#[derive(Serialize)]
pub(crate) struct XmlClass {
    #[serde(rename = "@id")]
    id: i32,

    #[serde(rename = "@days")]
    days: String,

    #[serde(rename = "@weeks")]
    weeks: String,

    #[serde(rename = "@start")]
    start: u32,

    #[serde(rename = "@room", skip_serializing_if = "Option::is_none")]
    room: Option<i32>,

    #[serde(rename = "student")]
    students: Vec<XmlStudent>,
}

impl XmlClass {
    pub(crate) fn from_domain(c: &Class, ctx: &OutputMetadata) -> Self {
        Self {
            id: c.id.0,
            days: bit_string_u8(c.days.0, ctx.nr_days),
            weeks: bit_string_u16(c.weeks.0, ctx.nr_weeks),
            start: c.start,
            room: c.room.map(|r| r.0),
            students: c
                .students
                .iter()
                .map(|s| XmlStudent { id: s.id.0 })
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct XmlStudent {
    #[serde(rename = "@id")]
    id: i32,
}
