use parser::{
    Problem, courses::ClassId, days::Days, rooms::RoomId, students::StudentId, weeks::Weeks,
};
use serde::Serialize;

use crate::dtos::{XmlClass, XmlOutput};

#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    // pub metadata: OutputMetadata,
    pub classes: Vec<Class>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OutputMetadata {
    pub name: String,
    pub runtime: f32,
    pub cores: usize,
    pub technique: String,
    pub author: String,
    pub institution: String,
    pub country: String,
    pub nr_days: u32,
    pub nr_weeks: u32,
}

impl OutputMetadata {
    pub fn from_problem(problem: &Problem) -> Self {
        Self {
            name: problem.name.clone(),
            runtime: 1.0,
            cores: 1,
            technique: "Genetic Algorithm".into(),
            author: "todo".into(),
            institution: "todo".into(),
            country: "todo".into(),
            nr_days: problem.nr_days,
            nr_weeks: problem.nr_weeks,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    pub id: ClassId,
    pub days: Days,
    pub weeks: Weeks,
    pub start: u32,
    pub room: Option<RoomId>,
    pub students: Vec<Student>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Student {
    pub id: StudentId,
}

impl Output {
    pub fn serialize(&self, context: OutputMetadata) -> String {
        let xml = XmlOutput {
            name: context.name.clone(),
            runtime: context.runtime,
            cores: context.cores,
            technique: context.technique.clone(),
            author: context.author.clone(),
            institution: context.institution.clone(),
            country: context.country.clone(),
            classes: self
                .classes
                .iter()
                .map(|c| XmlClass::from_domain(c, &context))
                .collect(),
        };
        let mut s = String::new();

        s.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        s.push('\n');
        s.push_str(
            r#"<!DOCTYPE solution PUBLIC
	        "-//ITC 2019//DTD Problem Format/EN"
	        "http://www.itc2019.org/competition-format.dtd">"#,
        );
        s.push('\n');

        s.push_str(&quick_xml::se::to_string(&xml).unwrap());
        s
    }
}
