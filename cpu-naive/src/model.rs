use std::collections::HashMap;

use parser::{
    courses::{ClassId, ConfigId, CourseId, SubpartId},
    distributions::DistributionKind,
    optimization::Optimization,
    problem::Problem,
    rooms::RoomId,
    students::StudentId,
    timeslots::TimeSlots,
};

const ID_BOUND: usize = 32768; // not the tightest upper bound

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

#[derive(Debug, Clone)]
pub struct RoomData {
    pub id: RoomId,
    pub capacity: u32,
    pub travels: Vec<TravelData>,
    pub unavailabilities: Vec<TimeSlots>,
}

#[derive(Debug, Clone)]
pub struct TravelData {
    /// index into [TimetableData::rooms]
    pub dest_room_idx: usize,
    pub travel_time: u32,
}

#[derive(Debug, Clone)]
pub struct StudentData {
    pub id: StudentId,
    /// indices into [TimetableData::courses]
    pub course_indices: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct DistributionData {
    pub kind: DistributionKind,
    pub class_indices: Vec<usize>,
    /// Some(x) = soft penalty x, None = hard penalty (see [Self::is_required])
    pub penalty: Option<u32>,
}

/// <...>_start and <...>_end are indices into the corresponding vector in TimetableData
/// *non-inclusive*
#[derive(Debug, Clone)]
pub struct CourseData {
    pub id: CourseId,
    pub configs_start: usize,
    pub configs_end: usize,
}

#[derive(Debug, Clone)]
pub struct ConfigData {
    pub id: ConfigId,
    pub subparts_start: usize,
    pub subparts_end: usize,
}

#[derive(Debug, Clone)]
pub struct SubpartData {
    pub id: SubpartId,
    pub classes_start: usize,
    pub classes_end: usize,
}

#[derive(Debug, Clone)]
pub struct ClassData {
    pub id: ClassId,
    /// None = no limit on number of students
    pub limit: Option<u32>,
    /// None = this class has no parent
    pub parent: Option<usize>,
    /// indices into [TimetableData::time_options]
    pub times_start: usize,
    pub times_end: usize,
    /// indices into [TimetableData::room_options]
    /// `rooms_start == rooms_end` means the class doesn't need a room.
    pub rooms_start: usize,
    pub rooms_end: usize,
    /// index into [TimetableData::subparts] for faster access
    pub subpart_id: usize,
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
    pub room_idx: usize,
    pub penalty: u32,
}

impl TimetableData {
    /// flattens the problem structure into a bunch of arrays
    pub fn new(p: Problem) -> Self {
        let room_id_to_idx: HashMap<_, _> = p
            .rooms
            .0
            .iter()
            .enumerate()
            .map(|(idx, r)| (r.id, idx))
            .collect();
        let course_id_to_idx: HashMap<_, _> = p
            .courses
            .0
            .iter()
            .enumerate()
            .map(|(idx, c)| (c.id, idx))
            .collect();
        let class_id_to_idx: HashMap<_, _> = p
            .courses
            .0
            .iter()
            .flat_map(|c| c.configs.clone())
            .flat_map(|c| c.subparts)
            .flat_map(|s| s.classes)
            .enumerate()
            .map(|(idx, c)| (c.id, idx))
            .collect();

        let rooms: Vec<_> = p
            .rooms
            .0
            .into_iter()
            .map(|r| {
                let travels: Vec<_> = r
                    .travels
                    .iter()
                    .map(|t| TravelData::new(room_id_to_idx[&t.room], t.value))
                    .collect();
                RoomData::new(r.id, r.capacity, travels, r.unavailabilities)
            })
            .collect();

        let students: Vec<_> = p
            .students
            .0
            .into_iter()
            .map(|s| {
                StudentData::new(
                    s.id,
                    s.courses
                        .into_iter()
                        .filter_map(|id| course_id_to_idx.get(&id).cloned())
                        .collect(),
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
                    d.classes
                        .into_iter()
                        .filter_map(|id| class_id_to_idx.get(&id).cloned())
                        .collect(),
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
                        room_options.extend(class.rooms.into_iter().filter_map(|r| {
                            room_id_to_idx
                                .get(&r.room)
                                .map(|&idx| RoomOption::new(idx, r.penalty))
                        }));
                        let rooms_end = room_options.len();

                        classes.push(ClassData::new(
                            class.id,
                            class.limit,
                            class.parent.map(|p| class_id_to_idx[&p]),
                            times_start,
                            times_end,
                            rooms_start,
                            rooms_end,
                            subparts.len(),
                        ));
                    }
                    let classes_end = classes.len();
                    subparts.push(SubpartData::new(subpart.id, classes_start, classes_end));
                }
                let subparts_end = subparts.len();
                configs.push(ConfigData::new(config.id, subparts_start, subparts_end));
            }
            let configs_end = configs.len();
            courses.push(CourseData::new(course.id, configs_start, configs_end));
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

impl RoomData {
    pub fn new(
        id: RoomId,
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
    pub fn new(dest_room_idx: usize, travel_time: u32) -> Self {
        Self {
            dest_room_idx,
            travel_time,
        }
    }
}

impl StudentData {
    pub fn new(id: StudentId, course_indices: Vec<usize>) -> Self {
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
    pub fn new(id: CourseId, configs_start: usize, configs_end: usize) -> Self {
        Self {
            id,
            configs_start,
            configs_end,
        }
    }
}

impl ConfigData {
    pub fn new(id: ConfigId, subparts_start: usize, subparts_end: usize) -> Self {
        Self {
            id,
            subparts_start,
            subparts_end,
        }
    }
}

impl SubpartData {
    pub fn new(id: SubpartId, classes_start: usize, classes_end: usize) -> Self {
        Self {
            id,
            classes_start,
            classes_end,
        }
    }
}

impl ClassData {
    pub fn new(
        id: ClassId,
        limit: Option<u32>,
        parent: Option<usize>,
        times_start: usize,
        times_end: usize,
        rooms_start: usize,
        rooms_end: usize,
        subpart_id: usize,
    ) -> Self {
        Self {
            id,
            limit,
            parent,
            times_start,
            times_end,
            rooms_start,
            rooms_end,
            subpart_id,
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
    pub fn new(room_idx: usize, penalty: u32) -> Self {
        Self { room_idx, penalty }
    }
}

impl DistributionData {
    pub fn is_required(&self) -> bool {
        self.penalty.is_none()
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
        assert_eq!(data.classes[parent_idx].id, ClassId::new(1));
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
        dbg!(&data);
        let class = &data.classes[2 - 1];
        assert_eq!(class.times_end - class.times_start, 2);
        // there is a room with id=4 in class2, but that room doesn't exist
        assert_eq!(class.rooms_end - class.rooms_start, 0);
        assert!(!data.classes[3 - 1].needs_room());
    }
}
