use std::ops::AddAssign;
use crate::fitness::Fitness;
use crate::model::TimetableData;
use crate::solution::Solution;

pub(crate) struct Distribution<'a> {
    data: &'a TimetableData,
    sol: &'a Solution,
}

impl<'a> Distribution<'a> {
    pub fn new(data: &'a TimetableData, sol: &'a Solution) -> Self {
        Self { data, sol }
    }

    /// returns [Fitness], because there can be both soft and hard constraints
    pub fn calculate_penalty(self) -> Fitness {
        let mut fitness = Fitness::new();

        fitness += self.same_start();
        fitness += self.same_time();
        fitness += self.different_time();
        fitness += self.same_days();
        fitness += self.different_days();
        fitness += self.same_weeks();
        fitness += self.different_weeks();
        fitness += self.overlap();
        fitness += self.not_overlap();
        fitness += self.same_room();
        fitness += self.different_room();
        fitness += self.same_attendees();
        fitness += self.precedence();
        fitness += self.work_day();
        fitness += self.min_gap();
        fitness += self.max_days();
        fitness += self.max_day_load();
        fitness += self.max_breaks();
        fitness += self.max_block();

        fitness
    }

    fn same_start(&self) -> Fitness {
        todo!()
    }

    fn same_time(&self) -> Fitness {
        todo!()
    }

    fn different_time(&self) -> Fitness {
        todo!()
    }

    fn same_days(&self) -> Fitness {
        todo!()
    }

    fn different_days(&self) -> Fitness {
        todo!()
    }

    fn same_weeks(&self) -> Fitness {
        todo!()
    }

    fn different_weeks(&self) -> Fitness {
        todo!()
    }

    fn overlap(&self) -> Fitness {
        todo!()
    }

    fn not_overlap(&self) -> Fitness {
        todo!()
    }

    fn same_room(&self) -> Fitness {
        todo!()
    }

    fn different_room(&self) -> Fitness {
        todo!()
    }

    fn same_attendees(&self) -> Fitness {
        todo!()
    }

    fn precedence(&self) -> Fitness {
        todo!()
    }

    fn work_day(&self) -> Fitness {
        todo!()
    }

    fn min_gap(&self) -> Fitness {
        todo!()
    }

    fn max_days(&self) -> Fitness {
        todo!()
    }

    fn max_day_load(&self) -> Fitness {
        todo!()
    }

    fn max_breaks(&self) -> Fitness {
        todo!()
    }

    fn max_block(&self) -> Fitness {
        todo!()
    }
}