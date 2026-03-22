use rand::{Rng, RngExt};

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
}

/// one particular assignment of classes to meetings times and rooms
#[derive(Debug, Clone)]
pub struct Solution {
    /// time slot assignments
    /// times[i] = assignment for the i-th class
    pub times: Vec<TimeOption>,
    /// room assignments
    /// rooms[i] = assignment for the i-th class,
    /// None if the class doesn't require a room
    pub rooms: Vec<Option<RoomOption>>,
}

#[derive(Debug, Clone)]
pub struct RoomData {
    pub id: usize,
    pub capacity: u32,
    pub travels: Vec<TravelData>,
    pub unavailabilities: Vec<TimeSlots>,
}

#[derive(Debug, Clone)]
pub struct TravelData {
    /// index into [TimetableData::rooms]
    pub dest_room_id: usize,
    pub travel_time: u32,
}

#[derive(Debug, Clone)]
pub struct StudentData {
    pub id: usize,
    /// indices into [TimetableData::courses]
    pub course_indices: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct DistributionData {
    pub kind: DistributionKind,
    pub class_indices: Vec<usize>,
    /// Some(x) = soft penalty x, None = hard penalty
    pub penalty: Option<u32>,
}

/// <...>_start and <...>_end are indices into the corresponding vector in TimetableData
/// *non-inclusive*
#[derive(Debug, Clone)]
pub struct CourseData {
    pub id: usize,
    pub configs_start: usize,
    pub configs_end: usize,
}

#[derive(Debug, Clone)]
pub struct ConfigData {
    pub id: usize,
    pub subparts_start: usize,
    pub subparts_end: usize,
}

#[derive(Debug, Clone)]
pub struct SubpartData {
    pub id: usize,
    pub classes_start: usize,
    pub classes_end: usize,
}

#[derive(Debug, Clone)]
pub struct ClassData {
    pub id: usize,
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

/// describes when a class will meet
#[derive(Debug, Clone)]
pub struct TimeOption {
    pub times: TimeSlots,
    pub penalty: u32,
}

/// describes where a class will meet
#[derive(Debug, Clone)]
pub struct RoomOption {
    /// index into [TimetableData::rooms]
    pub room_id: usize,
    pub penalty: u32,
}

impl TimetableData {
    /// flattens the problem structure into a bunch of arrays
    /// ALL IDS OF EVERYTHING ARE DECREMENTED BY 1 so that they can be directly used as array indexes,
    /// remember about incrementing them back when returning the solution
    pub fn new(p: Problem) -> Self {
        let rooms: Vec<_> = p
            .rooms
            .0
            .into_iter()
            .map(|r| {
                let travels: Vec<_> = r
                    .travels
                    .iter()
                    .map(|t| TravelData::new(t.room.0 - 1, t.value))
                    .collect();

                RoomData::new(r.id.0 - 1, r.capacity, travels, r.unavailabilities)
            })
            .collect();

        let students: Vec<_> = p
            .students
            .0
            .into_iter()
            .map(|s| {
                StudentData::new(
                    s.id.0 - 1,
                    s.courses.into_iter().map(|id| id.0 - 1).collect(),
                )
            })
            .collect();

        let distributions: Vec<_> = p
            .distributions
            .0
            .into_iter()
            .map(|d| {
                DistributionData::new(
                    d.kind,
                    d.classes.into_iter().map(|id| id.0 - 1).collect(),
                    d.penalty,
                )
            })
            .collect();

        let mut courses = Vec::new();
        let mut configs = Vec::new();
        let mut subparts = Vec::new();
        let mut classes = Vec::new();
        let mut time_options = Vec::new();
        let mut room_options = Vec::new();
        for course in p.courses.0 {
            let configs_start = configs.len();
            for config in course.configs {
                let subparts_start = subparts.len();
                for subpart in config.subparts {
                    let classes_start = classes.len();
                    for class in subpart.classes {
                        let times_start = time_options.len();
                        time_options.extend(
                            class
                                .times
                                .into_iter()
                                .map(|t| TimeOption::new(t.times, t.penalty)),
                        );
                        let times_end = time_options.len();

                        let rooms_start = room_options.len();
                        room_options.extend(
                            class
                                .rooms
                                .into_iter()
                                .map(|r| RoomOption::new(r.room.0 - 1, r.penalty)),
                        );
                        let rooms_end = room_options.len();

                        classes.push(ClassData::new(
                            class.id.0 - 1,
                            class.limit,
                            class.parent.map(|p| p.0 - 1),
                            times_start,
                            times_end,
                            rooms_start,
                            rooms_end,
                        ));
                    }
                    let classes_end = classes.len();
                    subparts.push(SubpartData::new(
                        subpart.id.0 - 1,
                        classes_start,
                        classes_end,
                    ));
                }
                let subparts_end = subparts.len();
                configs.push(ConfigData::new(
                    config.id.0 - 1,
                    subparts_start,
                    subparts_end,
                ));
            }
            let configs_end = configs.len();
            courses.push(CourseData::new(course.id.0 - 1, configs_start, configs_end));
        }

        Self {
            n_days: p.nr_days,
            n_weeks: p.nr_weeks,
            slots_per_day: p.slots_per_day,
            optimization: p.optimization,
            rooms,
            courses,
            configs,
            subparts,
            classes,
            time_options,
            room_options,
            distributions,
            students,
        }
    }
}

impl Solution {
    /// generates a random (quite possibly useless) solution
    /// by assigning to each class a random time slot and a random room
    /// out of its TimeOptions and RoomOptions
    pub fn new(data: &TimetableData, rng: &mut impl Rng) -> Self {
        let times: Vec<_> = data
            .classes
            .iter()
            .map(|class| {
                let i = rng.random_range(class.times_start..class.times_end);
                data.time_options[i].clone()
            })
            .collect();
        let rooms: Vec<_> = data
            .classes
            .iter()
            .map(|class| {
                class.needs_room().then(|| {
                    let i = rng.random_range(class.rooms_start..class.rooms_end);
                    data.room_options[i].clone()
                })
            })
            .collect();

        Self { times, rooms }
    }
}

impl RoomData {
    pub fn new(
        id: usize,
        capacity: u32,
        travels: Vec<TravelData>,
        unavailabilities: Vec<TimeSlots>,
    ) -> Self {
        Self {
            id,
            capacity,
            travels,
            unavailabilities,
        }
    }
}

impl TravelData {
    pub fn new(dest_room_id: usize, travel_time: u32) -> Self {
        Self {
            dest_room_id,
            travel_time,
        }
    }
}

impl StudentData {
    pub fn new(id: usize, course_indices: Vec<usize>) -> Self {
        Self { id, course_indices }
    }
}

impl DistributionData {
    pub fn new(kind: DistributionKind, class_indices: Vec<usize>, penalty: Option<u32>) -> Self {
        Self {
            kind,
            class_indices,
            penalty,
        }
    }
}

impl CourseData {
    pub fn new(id: usize, configs_start: usize, configs_end: usize) -> Self {
        Self {
            id,
            configs_start,
            configs_end,
        }
    }
}

impl ConfigData {
    pub fn new(id: usize, subparts_start: usize, subparts_end: usize) -> Self {
        Self {
            id,
            subparts_start,
            subparts_end,
        }
    }
}

impl SubpartData {
    pub fn new(id: usize, classes_start: usize, classes_end: usize) -> Self {
        Self {
            id,
            classes_start,
            classes_end,
        }
    }
}

impl ClassData {
    pub fn new(
        id: usize,
        limit: Option<u32>,
        parent: Option<usize>,
        times_start: usize,
        times_end: usize,
        rooms_start: usize,
        rooms_end: usize,
    ) -> Self {
        Self {
            id,
            limit,
            parent,
            times_start,
            times_end,
            rooms_start,
            rooms_end,
        }
    }

    pub fn needs_room(&self) -> bool {
        self.rooms_start != self.rooms_end
    }
}

impl TimeOption {
    pub fn new(times: TimeSlots, penalty: u32) -> Self {
        Self { times, penalty }
    }
}

impl RoomOption {
    pub fn new(room_id: usize, penalty: u32) -> Self {
        Self { room_id, penalty }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> TimetableData {
        let xml = include_str!("../../data/itc2019/sample.xml");
        let problem = Problem::parse(xml).unwrap();

        TimetableData::new(problem)
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
    fn has_parent() {
        let data = sample_data();
        let parent_idx = data.classes[3 - 1].parent.unwrap();
        assert_eq!(data.classes[parent_idx].id, 1 - 1);
    }

    #[test]
    #[should_panic]
    fn no_parent() {
        let data = sample_data();
        let _ = data.classes[2 - 1].parent.unwrap();
    }

    #[test]
    fn time_and_room_ranges() {
        let data = sample_data();
        let class = &data.classes[2 - 1];
        assert_eq!(class.times_end - class.times_start, 2);
        assert_eq!(class.rooms_end - class.rooms_start, 1);
        assert!(!data.classes[3 - 1].needs_room());
    }
}
