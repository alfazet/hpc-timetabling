use parser::distributions::DistributionKind;
use parser::timeslots::TimeSlots;
use crate::fitness::Fitness;
use crate::model::{DistributionData, RoomOption, TimetableData};
use crate::solution::Solution;

pub(crate) struct Distribution<'a> {
    data: &'a TimetableData,
    sol: &'a Solution,
}

impl Fitness {
    fn apply_penalty(&mut self, penalty: &Option<u32>) {
        match penalty {
            Some(penalty) => self.soft += penalty,
            None => self.hard += 1,
        }
    }
}

impl<'a> Distribution<'a> {
    pub fn new(data: &'a TimetableData, sol: &'a Solution) -> Self {
        Self { data, sol }
    }

    /// returns [Fitness], because there can be both soft and hard constraints
    pub fn calculate_penalty(self) -> Fitness {
        let mut fitness = Fitness::new();

        self.data.distributions.iter().for_each(|d| {
            fitness += match d.kind {
                DistributionKind::SameStart => self.same_start(d),
                DistributionKind::SameTime => self.same_time(d),
                DistributionKind::DifferentTime => self.different_time(d),
                DistributionKind::SameDays => self.same_days(d),
                DistributionKind::DifferentDays => self.different_days(d),
                DistributionKind::SameWeeks => self.same_weeks(d),
                DistributionKind::DifferentWeeks => self.different_weeks(d),
                DistributionKind::Overlap => self.overlap(d),
                DistributionKind::NotOverlap => self.not_overlap(d),
                DistributionKind::SameRoom => self.same_room(d),
                DistributionKind::DifferentRoom => self.different_room(d),
                DistributionKind::SameAttendees => self.same_attendees(d),
                DistributionKind::Precedence => self.precedence(d),
                DistributionKind::WorkDay(_) => self.work_day(d),
                DistributionKind::MinGap(_) => self.min_gap(d),
                DistributionKind::MaxDays(_) => self.max_days(d),
                DistributionKind::MaxDayLoad(_) => self.max_day_load(d),
                DistributionKind::MaxBreaks(_, _) => self.max_breaks(d),
                DistributionKind::MaxBlock(_, _) => self.max_block(d),
            }
        });

        fitness
    }

    fn same_start(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices.iter().enumerate().for_each(|(index, class_index)| {
            for i in index + 1..dist.class_indices.len() {
                let i_class_index = dist.class_indices[i];
                if self.sol.times[*class_index].times.start != self.sol.times[i_class_index].times.start {
                    fitness.apply_penalty(&dist.penalty);
                }
            }
        });

        fitness
    }

    fn same_time(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices.iter().enumerate().for_each(|(index, class_index)| {
            let class = &self.sol.times[*class_index].times;
            for i in index + 1..dist.class_indices.len() {
                let i_class = &self.sol.times[dist.class_indices[i]].times;
                if !((i_class.start <= class.start && class.start + class.length <= i_class.start + i_class.length)
                    || (class.start <= i_class.start && i_class.start + i_class.length <= class.start + class.length)) {
                    fitness.apply_penalty(&dist.penalty);
                }
            }
        });

        fitness
    }

    fn different_time(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices.iter().enumerate().for_each(|(index, class_index)| {
            let class = &self.sol.times[*class_index].times;
            for i in index + 1..dist.class_indices.len() {
                let i_class = &self.sol.times[dist.class_indices[i]].times;
                if !((i_class.start + i_class.length <= class.start)
                    || (class.start + class.length <= i_class.start)) {
                    fitness.apply_penalty(&dist.penalty);
                }
            }
        });

        fitness
    }

    fn same_days(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices.iter().enumerate().for_each(|(index, class_index)| {
            let class = &self.sol.times[*class_index].times.days;
            for i in index + 1..dist.class_indices.len() {
                let i_class = &self.sol.times[dist.class_indices[i]].times.days;
                if !(((i_class.0 | class.0) == i_class.0)
                    || ((i_class.0 | class.0) == class.0)) {
                    fitness.apply_penalty(&dist.penalty);
                }
            }
        });

        fitness
    }

    fn different_days(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices.iter().enumerate().for_each(|(index, class_index)| {
            let class = &self.sol.times[*class_index].times.days;
            for i in index + 1..dist.class_indices.len() {
                let i_class = &self.sol.times[dist.class_indices[i]].times.days;
                if (i_class.0 & class.0) != 0 {
                    fitness.apply_penalty(&dist.penalty);
                }
            }
        });

        fitness
    }

    fn same_weeks(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices.iter().enumerate().for_each(|(index, class_index)| {
            let class = &self.sol.times[*class_index].times.weeks;
            for i in index + 1..dist.class_indices.len() {
                let i_class = &self.sol.times[dist.class_indices[i]].times.weeks;
                if !(((i_class.0 | class.0) == i_class.0)
                    || ((i_class.0 | class.0) == class.0)) {
                    fitness.apply_penalty(&dist.penalty);
                }
            }
        });

        fitness
    }

    fn different_weeks(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices.iter().enumerate().for_each(|(index, class_index)| {
            let class = &self.sol.times[*class_index].times.weeks;
            for i in index + 1..dist.class_indices.len() {
                let i_class = &self.sol.times[dist.class_indices[i]].times.weeks;
                if (i_class.0 & class.0) != 0 {
                    fitness.apply_penalty(&dist.penalty);
                }
            }
        });

        fitness
    }

    fn does_overlap(c_i: &TimeSlots, c_j: &TimeSlots) -> bool {
        (c_j.start < c_i.start + c_i.length) && (c_i.start < c_j.start + c_j.length)
            && ((c_i.days.0 & c_j.days.0) != 0) && ((c_i.weeks.0 & c_j.weeks.0) != 0)
    }

    fn overlap(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices.iter().enumerate().for_each(|(index, class_index)| {
            let class = &self.sol.times[*class_index].times;
            for i in index + 1..dist.class_indices.len() {
                let i_class = &self.sol.times[dist.class_indices[i]].times;
                if !Self::does_overlap(class, i_class) {
                    fitness.apply_penalty(&dist.penalty);
                }
            }
        });

        fitness
    }

    fn not_overlap(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices.iter().enumerate().for_each(|(index, class_index)| {
            let class = &self.sol.times[*class_index].times;
            for i in index + 1..dist.class_indices.len() {
                let i_class = &self.sol.times[dist.class_indices[i]].times;
                if Self::does_overlap(class, i_class) {
                    fitness.apply_penalty(&dist.penalty);
                }
            }
        });

        fitness
    }

    fn in_same_room(r1: Option<&RoomOption>, r2: Option<&RoomOption>) -> bool {
        match r1 {
            Some(class_room_option) => match r2 {
                None => false,
                Some(i_class_room_option) =>
                    class_room_option.room_idx == i_class_room_option.room_idx,
            }
            None => match r2 {
                None => true,
                Some(_) => false,
            }
        }
    }

    fn same_room(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices.iter().enumerate().for_each(|(index, class_index)| {
            let class = (&self.sol.rooms[*class_index]).as_ref();
            (index + 1..dist.class_indices.len()).for_each(|i| {
                let i_class = (&self.sol.rooms[dist.class_indices[i]]).as_ref();
                if !Self::in_same_room(class, i_class) {
                    fitness.apply_penalty(&dist.penalty);
                }
            });
        });

        fitness
    }

    fn different_room(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices.iter().enumerate().for_each(|(index, class_index)| {
            let class = (&self.sol.rooms[*class_index]).as_ref();
            (index + 1..dist.class_indices.len()).for_each(|i| {
                let i_class = (&self.sol.rooms[dist.class_indices[i]]).as_ref();
                if Self::in_same_room(class, i_class) {
                    fitness.apply_penalty(&dist.penalty);
                }
            });
        });

        fitness
    }

    fn same_attendees(&self, dist: &DistributionData) -> Fitness {
        todo!()
    }

    fn precedence(&self, dist: &DistributionData) -> Fitness {
        todo!()
    }

    fn work_day(&self, dist: &DistributionData) -> Fitness {
        todo!()
    }

    fn min_gap(&self, dist: &DistributionData) -> Fitness {
        todo!()
    }

    fn max_days(&self, dist: &DistributionData) -> Fitness {
        todo!()
    }

    fn max_day_load(&self, dist: &DistributionData) -> Fitness {
        todo!()
    }

    fn max_breaks(&self, dist: &DistributionData) -> Fitness {
        todo!()
    }

    fn max_block(&self, dist: &DistributionData) -> Fitness {
        todo!()
    }
}