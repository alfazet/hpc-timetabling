use rand::{Rng, RngExt};
use std::collections::HashMap;

use parser::{
    distributions::DistributionKind, optimization::Optimization, problem::Problem,
    timeslots::TimeSlots,
};

#[derive(Debug, Clone)]
pub struct TimetableData {
    pub n_days: u32,
    pub n_weeks: u32,
    pub slots_per_day: u32,
    pub optimization: Optimization,

    pub rooms: Vec<RoomData>,
    pub courses: Vec<CourseData>,
    pub configs: Vec<ConfigData>,
    pub subparts: Vec<SubpartData>,
    pub classes: Vec<ClassData>,

    pub time_options: Vec<TimeOption>,
    pub room_options: Vec<RoomOption>,
    pub distributions: Vec<DistributionData>,
    pub students: Vec<StudentData>,

    pub class_id_to_idx: HashMap<i32, usize>,
}

#[derive(Debug, Clone)]
pub struct TravelData {
    /// index into [TimetableData::rooms]
    pub dest_room_idx: usize,
    pub travel_time: u32,
}

#[derive(Debug, Clone)]
pub struct RoomData {
    pub id: i32,
    pub capacity: u32,
    pub travels: Vec<TravelData>,
    pub unavailabilities: Vec<TimeSlots>,
}

/// <...>_start and <...>_end are indices into the corresponding vector in TimetableData
/// *non-inclusive*
#[derive(Debug, Clone)]
pub struct CourseData {
    pub id: i32,
    pub configs_start: usize,
    pub configs_end: usize,
}

#[derive(Debug, Clone)]
pub struct ConfigData {
    pub id: i32,
    pub subparts_start: usize,
    pub subparts_end: usize,
}

#[derive(Debug, Clone)]
pub struct SubpartData {
    pub id: i32,
    pub classes_start: usize,
    pub classes_end: usize,
}

#[derive(Debug, Clone)]
pub struct ClassData {
    pub original_id: i32,
    /// None = no limit on number of students
    pub limit: Option<u32>,
    /// None = this class has no parent
    pub parent: Option<usize>,
    /// indices into TimetableData::time_options
    pub times_start: usize,
    pub times_end: usize,
    /// indices into TimetableData::room_options
    /// rooms_start == rooms_end means the class doesn't need a room.
    pub rooms_start: usize,
    pub rooms_end: usize,
}

#[derive(Debug, Clone)]
pub struct TimeOption {
    pub times: TimeSlots,
    pub penalty: i32,
}

#[derive(Debug, Clone)]
pub struct RoomOption {
    /// index into [TimetableData::rooms]
    pub room_idx: usize,
    pub penalty: i32,
}

#[derive(Debug, Clone)]
pub struct DistributionData {
    pub kind: DistributionKind,
    pub class_indices: Vec<usize>,
    /// Some(x) = soft penalty x, None = hard penalty
    pub penalty: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct StudentData {
    pub id: i32,
    /// indices into [TimetableData::courses]
    pub course_indices: Vec<usize>,
}

/// one particular assignment of courses (a unit in the genetic algorithm's population)
#[derive(Debug, Clone)]
pub struct Solution {
    /// i-th item of this vec = the choice for the i-th course in Timetable::courses
    pub course_choices: Vec<CourseChoice>,
}

#[derive(Debug, Clone)]
pub struct CourseChoice {
    /// index into this course's config range, offset from [CourseData::configs_start]
    pub config_offset: usize,
    /// one choice per subpart of the selected config.
    pub subpart_choices: Vec<SubpartChoice>,
}

#[derive(Debug, Clone)]
pub struct SubpartChoice {
    /// index into this subparts's class range, offset from [SubpartData::classes_start]
    pub class_offset: usize,
    /// index into the chosen class' time options range, offset from [ClassData::times_start]
    pub time_offset: usize,
    /// index into the chosen class' room options range, offset from [ClassData::rooms_start]
    /// None if the class requires no room
    pub room_offset: Option<usize>,
}

impl ClassData {
    pub fn needs_room(&self) -> bool {
        self.rooms_start != self.rooms_end
    }

    pub fn n_time_options(&self) -> usize {
        self.times_end - self.times_start
    }

    pub fn n_room_options(&self) -> usize {
        self.rooms_end - self.rooms_start
    }
}

impl DistributionData {
    pub fn is_required(&self) -> bool {
        self.penalty.is_none()
    }
}

impl TimetableData {
    /// flattens the problem structure from a tree to a bunch of arrays
    /// TODO: refactor into smaller functions
    pub fn new(p: &Problem) -> Self {
        let mut room_id_to_idx: HashMap<_, _> = p
            .rooms
            .0
            .iter()
            .enumerate()
            .map(|(idx, r)| (r.id.0, idx))
            .collect();

        let mut rooms: Vec<_> = p
            .rooms
            .0
            .iter()
            .map(|r| RoomData {
                id: r.id.0,
                capacity: r.capacity,
                travels: r
                    .travels
                    .iter()
                    .filter_map(|t| {
                        room_id_to_idx.get(&t.room.0).map(|&idx| TravelData {
                            dest_room_idx: idx,
                            travel_time: t.value,
                        })
                    })
                    .collect(),
                unavailabilities: r.unavailabilities.clone(),
            })
            .collect();

        let mut courses = Vec::new();
        let mut configs = Vec::new();
        let mut subparts = Vec::new();
        let mut classes = Vec::new();
        let mut time_options = Vec::new();
        let mut room_options = Vec::new();
        let mut class_id_to_idx = HashMap::new();
        for course in &p.courses.0 {
            let configs_start = configs.len();
            for config in &course.configs {
                let subparts_start = subparts.len();
                for subpart in &config.subparts {
                    let classes_start = classes.len();
                    for class in &subpart.classes {
                        class_id_to_idx.insert(class.id.0, classes.len());
                        let times_start = time_options.len();
                        for t in &class.times {
                            time_options.push(TimeOption {
                                times: t.times.clone(),
                                penalty: t.penalty,
                            });
                        }
                        let times_end = time_options.len();
                        let rooms_start = room_options.len();
                        for r in &class.rooms {
                            let room_idx = *room_id_to_idx.entry(r.room.0).or_insert_with(|| {
                                let idx = rooms.len();
                                rooms.push(RoomData {
                                    id: r.room.0,
                                    capacity: 0,
                                    travels: Vec::new(),
                                    unavailabilities: Vec::new(),
                                });

                                idx
                            });
                            room_options.push(RoomOption {
                                room_idx,
                                penalty: r.penalty,
                            });
                        }
                        let rooms_end = room_options.len();
                        classes.push(ClassData {
                            original_id: class.id.0,
                            limit: class.limit,
                            // parents are resolved below
                            parent: None,
                            times_start,
                            times_end,
                            rooms_start,
                            rooms_end,
                        });
                    }
                    let classes_end = classes.len();
                    subparts.push(SubpartData {
                        id: subpart.id.0,
                        classes_start,
                        classes_end,
                    });
                }
                let subparts_end = subparts.len();
                configs.push(ConfigData {
                    id: config.id.0,
                    subparts_start,
                    subparts_end,
                });
            }
            let configs_end = configs.len();
            courses.push(CourseData {
                id: course.id.0,
                configs_start,
                configs_end,
            });
        }

        // resolving parents
        for parsed_course in &p.courses.0 {
            for config in &parsed_course.configs {
                for subpart in &config.subparts {
                    for class in &subpart.classes {
                        if let Some(ref parent_id) = class.parent {
                            let class_idx = class_id_to_idx[&class.id.0];
                            let parent_idx = class_id_to_idx[&parent_id.0];
                            classes[class_idx].parent = Some(parent_idx);
                        }
                    }
                }
            }
        }

        let course_id_to_idx: HashMap<_, _> = p
            .courses
            .0
            .iter()
            .enumerate()
            .map(|(idx, c)| (c.id.0, idx))
            .collect();
        let students: Vec<_> = p
            .students
            .0
            .iter()
            .map(|s| StudentData {
                id: s.id.0,
                course_indices: s
                    .courses
                    .iter()
                    .filter_map(|course_id| course_id_to_idx.get(&course_id.0).copied())
                    .collect(),
            })
            .collect();

        let distributions: Vec<_> = p
            .distributions
            .0
            .iter()
            .map(|d| DistributionData {
                kind: d.kind.clone(),
                class_indices: d
                    .classes
                    .iter()
                    .filter_map(|class_id| class_id_to_idx.get(&class_id.0).copied())
                    .collect(),
                penalty: d.penalty,
            })
            .collect();

        Self {
            n_days: p.nr_days,
            n_weeks: p.nr_weeks,
            slots_per_day: p.slots_per_day,
            optimization: p.optimization.clone(),
            rooms,
            courses,
            configs,
            subparts,
            classes,
            time_options,
            room_options,
            distributions,
            students,
            class_id_to_idx,
        }
    }
}

impl Solution {
    /// generate a random, "valid" solution (valid as in "everything adheres to the requirements", not as in
    /// "there are no conflicts").
    pub fn new(data: &TimetableData, rng: &mut impl Rng) -> Self {
        let mut course_choices = Vec::new();
        for course in &data.courses {
            let n_configs = course.configs_end - course.configs_start;
            let config_offset = rng.random_range(0..n_configs);
            let config = &data.configs[course.configs_start + config_offset];
            let mut subpart_choices = Vec::new();
            for subpart_idx in config.subparts_start..config.subparts_end {
                let subpart = &data.subparts[subpart_idx];
                let n_classes = subpart.classes_end - subpart.classes_start;
                let class_offset = rng.random_range(0..n_classes);
                let class = &data.classes[subpart.classes_start + class_offset];
                let time_offset = rng.random_range(0..class.n_time_options());
                let room_offset = class
                    .needs_room()
                    .then(|| rng.random_range(0..class.n_room_options()));
                subpart_choices.push(SubpartChoice {
                    class_offset,
                    time_offset,
                    room_offset,
                });
            }
            course_choices.push(CourseChoice {
                config_offset,
                subpart_choices,
            });
        }

        Self { course_choices }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> TimetableData {
        let xml = include_str!("../../data/itc2019/sample.xml");
        let problem = Problem::parse(xml).unwrap();

        TimetableData::new(&problem)
    }

    #[test]
    fn from_problem_course_count() {
        let data = sample_data();
        assert_eq!(data.courses.len(), 1);
    }

    #[test]
    fn from_problem_config_count() {
        let data = sample_data();
        let course = &data.courses[0];
        assert_eq!(course.configs_end - course.configs_start, 1);
    }

    #[test]
    fn from_problem_subpart_count() {
        let data = sample_data();
        let config = &data.configs[0];
        assert_eq!(config.subparts_end - config.subparts_start, 2);
    }

    #[test]
    fn from_problem_class_count() {
        let data = sample_data();
        assert_eq!(data.classes.len(), 3);
    }

    #[test]
    fn parent_resolved() {
        let data = sample_data();
        let idx3 = data.class_id_to_idx[&3];
        let parent_idx = data.classes[idx3].parent.unwrap();
        assert_eq!(data.classes[parent_idx].original_id, 1);
    }

    #[test]
    fn time_and_room_ranges() {
        let data = sample_data();
        let idx1 = data.class_id_to_idx[&1];
        let class = &data.classes[idx1];
        assert_eq!(class.n_time_options(), 2);
        assert_eq!(class.n_room_options(), 2);
        let idx3 = data.class_id_to_idx[&3];
        assert!(!data.classes[idx3].needs_room());
    }

    #[test]
    fn random_solution_is_consistent() {
        let data = sample_data();
        let mut rng = rand::rng();
        let sol = Solution::new(&data, &mut rng);
        assert_eq!(sol.course_choices.len(), data.courses.len());
        for (cc_idx, course) in data.courses.iter().enumerate() {
            let choice = &sol.course_choices[cc_idx];
            let n_configs = course.configs_end - course.configs_start;
            assert!(choice.config_offset < n_configs);
            let config = &data.configs[course.configs_start + choice.config_offset];
            let n_subparts = config.subparts_end - config.subparts_start;
            assert_eq!(choice.subpart_choices.len(), n_subparts);

            for (sc_idx, subpart_choice) in choice.subpart_choices.iter().enumerate() {
                let subpart = &data.subparts[config.subparts_start + sc_idx];
                let n_classes = subpart.classes_end - subpart.classes_start;
                assert!(subpart_choice.class_offset < n_classes);
                let class = &data.classes[subpart.classes_start + subpart_choice.class_offset];
                assert!(subpart_choice.time_offset < class.n_time_options());
                if class.needs_room() {
                    assert!(subpart_choice.room_offset.unwrap() < class.n_room_options());
                } else {
                    assert!(subpart_choice.room_offset.is_none());
                }
            }
        }
    }
}
