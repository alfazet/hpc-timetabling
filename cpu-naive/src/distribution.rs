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

        fitness
    }

    fn same_start(&self) -> Fitness {
        todo!()
    }
}