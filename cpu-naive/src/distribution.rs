use crate::fitness::Fitness;
use crate::model::{DistributionData, RoomOption, TimetableData};
use crate::solution::Solution;
use parser::distributions::DistributionKind;
use parser::timeslots::TimeSlots;
use std::cmp::{max, min};
use std::collections::HashMap;

pub(crate) struct Distribution<'a> {
    data: &'a TimetableData,
    sol: &'a Solution,
}

impl Fitness {
    fn apply_penalty(&mut self, penalty: Option<u32>) {
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
                DistributionKind::WorkDay(s) => self.work_day(d, s),
                DistributionKind::MinGap(s) => self.min_gap(d, s),
                DistributionKind::MaxDays(s) => self.max_days(d, s),
                DistributionKind::MaxDayLoad(s) => self.max_day_load(d, s),
                DistributionKind::MaxBreaks(r, s) => self.max_breaks(d, r, s),
                DistributionKind::MaxBlock(m, s) => self.max_block(d, m, s),
            }
        });

        fitness
    }

    fn same_start(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                for i in index + 1..dist.class_indices.len() {
                    let i_class_index = dist.class_indices[i];
                    if self.sol.times[class_index].times.start
                        != self.sol.times[i_class_index].times.start
                    {
                        fitness.apply_penalty(dist.penalty);
                    }
                }
            });

        fitness
    }

    fn same_time(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times;
                    if !((i_class.start <= class.start
                        && class.start + class.length <= i_class.start + i_class.length)
                        || (class.start <= i_class.start
                            && i_class.start + i_class.length <= class.start + class.length))
                    {
                        fitness.apply_penalty(dist.penalty);
                    }
                }
            });

        fitness
    }

    fn different_time(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times;
                    if !((i_class.start + i_class.length <= class.start)
                        || (class.start + class.length <= i_class.start))
                    {
                        fitness.apply_penalty(dist.penalty);
                    }
                }
            });

        fitness
    }

    fn same_days(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times.days;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times.days;
                    if !(((i_class.0 | class.0) == i_class.0) || ((i_class.0 | class.0) == class.0))
                    {
                        fitness.apply_penalty(dist.penalty);
                    }
                }
            });

        fitness
    }

    fn different_days(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times.days;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times.days;
                    if (i_class.0 & class.0) != 0 {
                        fitness.apply_penalty(dist.penalty);
                    }
                }
            });

        fitness
    }

    fn same_weeks(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times.weeks;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times.weeks;
                    if !(((i_class.0 | class.0) == i_class.0) || ((i_class.0 | class.0) == class.0))
                    {
                        fitness.apply_penalty(dist.penalty);
                    }
                }
            });

        fitness
    }

    fn different_weeks(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times.weeks;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times.weeks;
                    if (i_class.0 & class.0) != 0 {
                        fitness.apply_penalty(dist.penalty);
                    }
                }
            });

        fitness
    }

    fn does_overlap(c_i: &TimeSlots, c_j: &TimeSlots) -> bool {
        (c_j.start < c_i.start + c_i.length)
            && (c_i.start < c_j.start + c_j.length)
            && ((c_i.days.0 & c_j.days.0) != 0)
            && ((c_i.weeks.0 & c_j.weeks.0) != 0)
    }

    fn overlap(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times;
                    if !Self::does_overlap(class, i_class) {
                        fitness.apply_penalty(dist.penalty);
                    }
                }
            });

        fitness
    }

    fn not_overlap(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times;
                    if Self::does_overlap(class, i_class) {
                        fitness.apply_penalty(dist.penalty);
                    }
                }
            });

        fitness
    }

    fn in_same_room(r1: Option<&RoomOption>, r2: Option<&RoomOption>) -> bool {
        match r1 {
            Some(class_room_option) => match r2 {
                None => false,
                Some(i_class_room_option) => {
                    class_room_option.room_idx == i_class_room_option.room_idx
                }
            },
            None => r2.is_none(),
        }
    }

    fn same_room(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = self.sol.rooms[class_index].as_ref();
                (index + 1..dist.class_indices.len()).for_each(|i| {
                    let i_class = self.sol.rooms[dist.class_indices[i]].as_ref();
                    if !Self::in_same_room(class, i_class) {
                        fitness.apply_penalty(dist.penalty);
                    }
                });
            });

        fitness
    }

    fn different_room(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = self.sol.rooms[class_index].as_ref();
                (index + 1..dist.class_indices.len()).for_each(|i| {
                    let i_class = self.sol.rooms[dist.class_indices[i]].as_ref();
                    if Self::in_same_room(class, i_class) {
                        fitness.apply_penalty(dist.penalty);
                    }
                });
            });

        fitness
    }

    fn same_attendees(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        for (index, &class_index) in dist.class_indices.iter().enumerate() {
            let class_room = self.sol.rooms[class_index].as_ref();
            let Some(class_room) = class_room else {
                continue;
            };

            let class_time = &self.sol.times[class_index].times;

            for i in index + 1..dist.class_indices.len() {
                let i_class_room = self.sol.rooms[dist.class_indices[i]].as_ref();
                let Some(i_class_room) = i_class_room else {
                    continue;
                };

                let i_class_time = &self.sol.times[dist.class_indices[i]].times;

                let days_overlap = i_class_time.days.0 & class_time.days.0 != 0;
                let weeks_overlap = i_class_time.days.0 & class_time.days.0 != 0;
                if !days_overlap || !weeks_overlap {
                    continue;
                }

                let travel_time = max(
                    self.data.rooms[i_class_room.room_idx]
                        .travels
                        .iter()
                        .find(|td| td.dest_room_idx == class_room.room_idx)
                        .map(|td| td.travel_time)
                        .unwrap_or(0),
                    self.data.rooms[class_room.room_idx]
                        .travels
                        .iter()
                        .find(|td| td.dest_room_idx == i_class_room.room_idx)
                        .map(|td| td.travel_time)
                        .unwrap_or(0),
                );

                if !((i_class_time.start + i_class_time.length + travel_time <= class_time.start)
                    || (class_time.start + class_time.length + travel_time <= i_class_time.start))
                {
                    fitness.apply_penalty(dist.penalty);
                }
            }
        }

        fitness
    }

    fn precedence(&self, dist: &DistributionData) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(i, &class_index)| {
                let c_i = &self.sol.times[class_index].times;
                (i + 1..dist.class_indices.len()).for_each(|j| {
                    // i < j
                    let c_j = &self.sol.times[dist.class_indices[j]].times;
                    if !((c_i.weeks.0.leading_zeros() < c_j.weeks.0.leading_zeros())
                        || ((c_i.weeks.0.leading_zeros() == c_j.weeks.0.leading_zeros())
                            && ((c_i.days.0.leading_zeros() < c_j.days.0.leading_zeros())
                                || ((c_i.days.0.leading_zeros() == c_j.days.0.leading_zeros())
                                    && (c_i.start + c_i.length <= c_j.start)))))
                    {
                        fitness.apply_penalty(dist.penalty);
                    }
                });
            });

        fitness
    }

    fn work_day(&self, dist: &DistributionData, max_slots: u16) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(i, &class_index)| {
                let c_i = &self.sol.times[class_index].times;
                (i + 1..dist.class_indices.len()).for_each(|j| {
                    let c_j = &self.sol.times[dist.class_indices[j]].times;
                    if !(((c_i.days.0 & c_j.days.0) == 0)
                        || ((c_i.weeks.0 & c_j.weeks.0) == 0)
                        || (max(c_i.start + c_i.length, c_j.start + c_j.length)
                            - min(c_i.start, c_j.start)
                            <= max_slots as u32))
                    {
                        fitness.apply_penalty(dist.penalty);
                    }
                });
            });

        fitness
    }

    fn min_gap(&self, dist: &DistributionData, min_gap: u16) -> Fitness {
        let mut fitness = Fitness::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(i, &class_index)| {
                let c_i = &self.sol.times[class_index].times;
                (i + 1..dist.class_indices.len()).for_each(|j| {
                    let c_j = &self.sol.times[dist.class_indices[j]].times;
                    if !(((c_i.days.0 & c_j.days.0) == 0)
                        || ((c_i.weeks.0 & c_j.weeks.0) == 0)
                        || (c_i.start + c_i.length + min_gap as u32 <= c_j.start)
                        || (c_j.start + c_j.length + min_gap as u32 <= c_i.start))
                    {
                        fitness.apply_penalty(dist.penalty);
                    }
                });
            });

        fitness
    }

    fn max_days(&self, dist: &DistributionData, max_days: u8) -> Fitness {
        let max_days = max_days as u32;
        let mut fitness = Fitness::new();

        let mut days = 0;
        dist.class_indices.iter().for_each(|&class_idx| {
            days |= self.sol.times[class_idx].times.days.0;
        });

        let nonzero_bits = days.count_ones();
        if nonzero_bits > max_days {
            fitness.apply_penalty(dist.penalty);
            fitness.soft *= nonzero_bits - max_days;
        }

        fitness
    }

    fn max_day_load(&self, dist: &DistributionData, s: u16) -> Fitness {
        let s = s as u32;
        let mut days: HashMap<(u8, u8), u32> = HashMap::new();

        for &class_idx in &dist.class_indices {
            let times = &self.sol.times[class_idx].times;

            for w in 0..self.data.n_weeks as u8 {
                for d in 0..self.data.n_days as u8 {
                    if !times.weeks.contains(w) || !times.days.contains(d) {
                        continue;
                    }

                    *days.entry((w, d)).or_insert(0) += times.length;
                }
            }
        }

        let mut factor = 0;
        let mut penalty = Fitness::new();
        for (_, day_load) in days {
            if day_load > s {
                factor = day_load - s;
                penalty.apply_penalty(dist.penalty);
            }
        }

        penalty.soft *= factor;
        penalty.soft /= self.data.n_weeks;
        penalty
    }

    fn max_breaks(&self, dist: &DistributionData, r: u16, s: u16) -> Fitness {
        todo!()
    }

    fn max_block(&self, dist: &DistributionData, m: u16, s: u16) -> Fitness {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn same_days() {
        
    }
}