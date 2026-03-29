use parser::distributions::DistributionKind;
use crate::fitness::Fitness;
use crate::model::{DistributionData, TimetableData};
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
        todo!()
    }

    fn same_weeks(&self, dist: &DistributionData) -> Fitness {
        todo!()
    }

    fn different_weeks(&self, dist: &DistributionData) -> Fitness {
        todo!()
    }

    fn overlap(&self, dist: &DistributionData) -> Fitness {
        todo!()
    }

    fn not_overlap(&self, dist: &DistributionData) -> Fitness {
        todo!()
    }

    fn same_room(&self, dist: &DistributionData) -> Fitness {
        todo!()
    }

    fn different_room(&self, dist: &DistributionData) -> Fitness {
        todo!()
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