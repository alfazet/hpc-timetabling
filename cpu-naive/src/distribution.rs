use crate::model::{DistributionData, RoomOption, TimetableData};
use crate::penalty::Penalty;
use crate::solution::Solution;
use parser::distributions::DistributionKind;
use parser::timeslots::TimeSlots;
use std::cmp::{max, min};
use std::collections::HashMap;

pub(crate) struct Distribution<'a> {
    data: &'a TimetableData,
    sol: &'a Solution,
}

impl Penalty {
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

    /// returns [Penalty], because there can be both soft and hard constraints
    pub fn calculate_penalty(&self) -> Penalty {
        let mut penalty = Penalty::new();

        self.data.distributions.iter().for_each(|d| {
            penalty += match d.kind {
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

        penalty
    }

    fn same_start(&self, dist: &DistributionData) -> Penalty {
        let mut penalty = Penalty::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                for i in index + 1..dist.class_indices.len() {
                    let i_class_index = dist.class_indices[i];
                    if self.sol.times[class_index].times.start
                        != self.sol.times[i_class_index].times.start
                    {
                        penalty.apply_penalty(dist.penalty);
                    }
                }
            });

        penalty
    }

    fn same_time(&self, dist: &DistributionData) -> Penalty {
        let mut penalty = Penalty::new();

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
                        penalty.apply_penalty(dist.penalty);
                    }
                }
            });

        penalty
    }

    fn different_time(&self, dist: &DistributionData) -> Penalty {
        let mut penalty = Penalty::new();

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
                        penalty.apply_penalty(dist.penalty);
                    }
                }
            });

        penalty
    }

    fn same_days(&self, dist: &DistributionData) -> Penalty {
        let mut penalty = Penalty::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times.days;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times.days;
                    if !(((i_class.0 | class.0) == i_class.0) || ((i_class.0 | class.0) == class.0))
                    {
                        penalty.apply_penalty(dist.penalty);
                    }
                }
            });

        penalty
    }

    fn different_days(&self, dist: &DistributionData) -> Penalty {
        let mut penalty = Penalty::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times.days;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times.days;
                    if (i_class.0 & class.0) != 0 {
                        penalty.apply_penalty(dist.penalty);
                    }
                }
            });

        penalty
    }

    fn same_weeks(&self, dist: &DistributionData) -> Penalty {
        let mut penalty = Penalty::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times.weeks;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times.weeks;
                    if !(((i_class.0 | class.0) == i_class.0) || ((i_class.0 | class.0) == class.0))
                    {
                        penalty.apply_penalty(dist.penalty);
                    }
                }
            });

        penalty
    }

    fn different_weeks(&self, dist: &DistributionData) -> Penalty {
        let mut penalty = Penalty::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times.weeks;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times.weeks;
                    if (i_class.0 & class.0) != 0 {
                        penalty.apply_penalty(dist.penalty);
                    }
                }
            });

        penalty
    }

    fn does_overlap(c_i: &TimeSlots, c_j: &TimeSlots) -> bool {
        (c_j.start < c_i.start + c_i.length)
            && (c_i.start < c_j.start + c_j.length)
            && ((c_i.days.0 & c_j.days.0) != 0)
            && ((c_i.weeks.0 & c_j.weeks.0) != 0)
    }

    fn overlap(&self, dist: &DistributionData) -> Penalty {
        let mut penalty = Penalty::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times;
                    if !Self::does_overlap(class, i_class) {
                        penalty.apply_penalty(dist.penalty);
                    }
                }
            });

        penalty
    }

    fn not_overlap(&self, dist: &DistributionData) -> Penalty {
        let mut penalty = Penalty::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = &self.sol.times[class_index].times;
                for i in index + 1..dist.class_indices.len() {
                    let i_class = &self.sol.times[dist.class_indices[i]].times;
                    if Self::does_overlap(class, i_class) {
                        penalty.apply_penalty(dist.penalty);
                    }
                }
            });

        penalty
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

    fn same_room(&self, dist: &DistributionData) -> Penalty {
        let mut penalty = Penalty::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = self.sol.rooms[class_index].as_ref();
                (index + 1..dist.class_indices.len()).for_each(|i| {
                    let i_class = self.sol.rooms[dist.class_indices[i]].as_ref();
                    if !Self::in_same_room(class, i_class) {
                        penalty.apply_penalty(dist.penalty);
                    }
                });
            });

        penalty
    }

    fn different_room(&self, dist: &DistributionData) -> Penalty {
        let mut penalty = Penalty::new();

        dist.class_indices
            .iter()
            .enumerate()
            .for_each(|(index, &class_index)| {
                let class = self.sol.rooms[class_index].as_ref();
                (index + 1..dist.class_indices.len()).for_each(|i| {
                    let i_class = self.sol.rooms[dist.class_indices[i]].as_ref();
                    if Self::in_same_room(class, i_class) {
                        penalty.apply_penalty(dist.penalty);
                    }
                });
            });

        penalty
    }

    fn same_attendees(&self, dist: &DistributionData) -> Penalty {
        let mut penalty = Penalty::new();

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
                    penalty.apply_penalty(dist.penalty);
                }
            }
        }

        penalty
    }

    fn precedence(&self, dist: &DistributionData) -> Penalty {
        let mut penalty = Penalty::new();

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
                        penalty.apply_penalty(dist.penalty);
                    }
                });
            });

        penalty
    }

    fn work_day(&self, dist: &DistributionData, max_slots: u16) -> Penalty {
        let mut penalty = Penalty::new();

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
                        penalty.apply_penalty(dist.penalty);
                    }
                });
            });

        penalty
    }

    fn min_gap(&self, dist: &DistributionData, min_gap: u16) -> Penalty {
        let mut penalty = Penalty::new();

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
                        penalty.apply_penalty(dist.penalty);
                    }
                });
            });

        penalty
    }

    fn max_days(&self, dist: &DistributionData, max_days: u8) -> Penalty {
        let max_days = max_days as u32;
        let mut penalty = Penalty::new();

        let mut days = 0;
        dist.class_indices.iter().for_each(|&class_idx| {
            days |= self.sol.times[class_idx].times.days.0;
        });

        let nonzero_bits = days.count_ones();
        if nonzero_bits > max_days {
            penalty.apply_penalty(dist.penalty);
            penalty.soft *= nonzero_bits - max_days;
        }

        penalty
    }

    fn max_day_load(&self, dist: &DistributionData, s: u16) -> Penalty {
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
        let mut penalty = Penalty::new();
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

    fn max_breaks(&self, dist: &DistributionData, r: u16, s: u16) -> Penalty {
        todo!()
    }

    fn max_block(&self, dist: &DistributionData, m: u16, s: u16) -> Penalty {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::distribution::Distribution;
    use crate::model::TimetableData;
    use crate::penalty::Penalty;
    use crate::solution::Solution;
    use parser::Problem;
    use std::sync::LazyLock;

    static DATA1: LazyLock<TimetableData, fn() -> TimetableData> = LazyLock::new(|| {
        TimetableData::new(
            Problem::parse(include_str!("../../data/test-data/distribution-test-1.xml")).unwrap(),
        )
    });

    #[test]
    fn same_start() {
        // both distributions violated
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[5].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 1, soft: 0 },
            dist.same_start(&dist.data.distributions[0])
        );
        assert_eq!(
            Penalty { hard: 0, soft: 15 },
            dist.same_start(&dist.data.distributions[1])
        );

        // both distributions satisfied
        let sol = Solution {
            times: vec![
                DATA1.time_options[1].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[6].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(Penalty::new(), dist.same_start(&dist.data.distributions[0]));
        assert_eq!(Penalty::new(), dist.same_start(&dist.data.distributions[1]));
    }

    #[test]
    fn same_time() {
        // first starts before second starts and ends before second ends -- violation
        // 1: |---|
        // 2:   |----|
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[4].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 0, soft: 10 },
            dist.same_time(&dist.data.distributions[2])
        );

        // first one fits into second one's timespan -- valid
        // 1:  |---|
        // 2: |--------|
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[6].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(Penalty::new(), dist.same_time(&dist.data.distributions[2]));

        // first starts after second starts and ends after second ends -- violation
        // 1:     |---|
        // 2:  |-----|
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[7].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 0, soft: 10 },
            dist.same_time(&dist.data.distributions[2])
        );
    }

    #[test]
    fn different_time() {
        // first starts and ends before second -- valid
        // 1: |---|
        // 2:       |-------|
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[5].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty::new(),
            dist.different_time(&dist.data.distributions[3])
        );

        // first starts before second starts and ends before second ends -- violation
        // 1: |---|
        // 2:   |----|
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[4].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 1, soft: 0 },
            dist.different_time(&dist.data.distributions[3])
        );

        // first one fits into second one's timespan -- violation
        // 1:  |---|
        // 2: |--------|
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[6].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 1, soft: 0 },
            dist.different_time(&dist.data.distributions[3])
        );

        // first starts after second starts and ends after second ends -- violation
        // 1:     |---|
        // 2:  |-----|
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[7].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 1, soft: 0 },
            dist.different_time(&dist.data.distributions[3])
        );

        // first starts and ends after second -- valid
        // 1:          |---|
        // 2: |-----|
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[8].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty::new(),
            dist.different_time(&dist.data.distributions[3])
        );
    }

    #[test]
    fn same_days() {
        // second's days are a subset of first's days -- valid
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[8].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(Penalty::new(), dist.same_days(&dist.data.distributions[4]));

        let sol = Solution {
            times: vec![
                DATA1.time_options[1].clone(),
                DATA1.time_options[3].clone(),
                DATA1.time_options[8].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(Penalty::new(), dist.same_days(&dist.data.distributions[4]));

        // some days are different -- violation
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[3].clone(),
                DATA1.time_options[8].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 0, soft: 15 },
            dist.same_days(&dist.data.distributions[4])
        );
    }

    #[test]
    fn different_days() {
        // three classes do not overlap (in days context) -- valid
        let sol = Solution {
            times: vec![
                DATA1.time_options[1].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[5].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty::new(),
            dist.different_days(&dist.data.distributions[5])
        );

        // overlapping classes -- violation
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[5].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 2, soft: 0 },
            dist.different_days(&dist.data.distributions[5])
        );

        let sol = Solution {
            times: vec![
                DATA1.time_options[1].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[8].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 2, soft: 0 },
            dist.different_days(&dist.data.distributions[5])
        );
    }

    #[test]
    fn same_weeks() {
        // valid
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[5].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(Penalty::new(), dist.same_weeks(&dist.data.distributions[6]));

        // invalid
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[4].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 0, soft: 20 },
            dist.same_weeks(&dist.data.distributions[6])
        );
    }

    #[test]
    fn different_weeks() {
        // invalid
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[5].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 0, soft: 20 },
            dist.different_weeks(&dist.data.distributions[7])
        );

        // valid
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[4].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty::new(),
            dist.different_weeks(&dist.data.distributions[7])
        );
    }

    #[test]
    fn overlap_and_not_overlap() {
        // overlap
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[2].clone(),
                DATA1.time_options[5].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(Penalty::new(), dist.overlap(&dist.data.distributions[8]));
        assert_eq!(
            Penalty { hard: 1, soft: 0 },
            dist.not_overlap(&dist.data.distributions[9])
        );

        // not overlap
        let sol = Solution {
            times: vec![
                DATA1.time_options[0].clone(),
                DATA1.time_options[3].clone(),
                DATA1.time_options[4].clone(),
            ],
            rooms: vec![], // unnecessary
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 1, soft: 0 },
            dist.overlap(&dist.data.distributions[8])
        );
        assert_eq!(
            Penalty::new(),
            dist.not_overlap(&dist.data.distributions[9])
        );
    }

    #[test]
    fn same_and_different_room() {
        // same room (2 classes) and no room (third class)
        let sol = Solution {
            times: vec![], // unnecessary
            rooms: vec![
                Some(DATA1.room_options[1].clone()),
                Some(DATA1.room_options[2].clone()),
                None,
            ],
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 2, soft: 0 },
            dist.same_room(&dist.data.distributions[10])
        );
        assert_eq!(
            Penalty { hard: 1, soft: 0 },
            dist.different_room(&dist.data.distributions[11])
        );

        // every class in unique room
        let sol = Solution {
            times: vec![], // unnecessary
            rooms: vec![
                Some(DATA1.room_options[0].clone()),
                Some(DATA1.room_options[2].clone()),
                None,
            ],
        };
        let dist = Distribution::new(&DATA1, &sol);
        assert_eq!(
            Penalty { hard: 3, soft: 0 },
            dist.same_room(&dist.data.distributions[10])
        );
        assert_eq!(
            Penalty::new(),
            dist.different_room(&dist.data.distributions[11])
        );
    }

    static DATA2: LazyLock<TimetableData, fn() -> TimetableData> = LazyLock::new(|| {
        TimetableData::new(
            Problem::parse(include_str!("../../data/test-data/distribution-test-2.xml")).unwrap(),
        )
    });

    #[test]
    fn same_attendees() {
        // valid solution, instructor able to attend every class
        let sol = Solution {
            times: vec![
                DATA2.time_options[0].clone(),
                DATA2.time_options[3].clone(),
                DATA2.time_options[5].clone(),
            ],
            rooms: vec![
                Some(DATA2.room_options[0].clone()),
                Some(DATA2.room_options[2].clone()),
                Some(DATA2.room_options[4].clone()),
            ],
        };
        let dist = Distribution::new(&DATA2, &sol);
        assert_eq!(
            Penalty::new(),
            dist.same_attendees(&dist.data.distributions[0])
        );

        // invalid solution, not enough time to travel from class 1 to class 3
        // (classes don't overlap though)
        let sol = Solution {
            times: vec![
                DATA2.time_options[0].clone(),
                DATA2.time_options[3].clone(),
                DATA2.time_options[5].clone(),
            ],
            rooms: vec![
                Some(DATA2.room_options[0].clone()),
                Some(DATA2.room_options[2].clone()),
                Some(DATA2.room_options[5].clone()),
            ],
        };
        let dist = Distribution::new(&DATA2, &sol);
        assert_eq!(
            Penalty { hard: 1, soft: 0 },
            dist.same_attendees(&dist.data.distributions[0])
        );

        // invalid, every class collide with each other
        let sol = Solution {
            times: vec![
                DATA2.time_options[0].clone(),
                DATA2.time_options[2].clone(),
                DATA2.time_options[8].clone(),
            ],
            rooms: vec![
                Some(DATA2.room_options[0].clone()),
                Some(DATA2.room_options[2].clone()),
                Some(DATA2.room_options[5].clone()),
            ],
        };
        let dist = Distribution::new(&DATA2, &sol);
        assert_eq!(
            Penalty { hard: 3, soft: 0 },
            dist.same_attendees(&dist.data.distributions[0])
        );
    }

    /*#[test]
    fn precedence() {

    }

    #[test]
    fn work_day() {

    }

    #[test]
    fn min_gap() {

    }

    #[test]
    fn max_days() {

    }

    #[test]
    fn max_day_load() {

    }

    #[test]
    fn max_breaks() {

    }

    #[test]
    fn max_block() {

    }*/
}
