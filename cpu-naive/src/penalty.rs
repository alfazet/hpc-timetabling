use std::ops::{Add, AddAssign};
use std::{cmp::Ordering, fmt::Display};

/// individual penalty as the number of hard and soft violations,
/// lower value indicates a better solution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Penalty {
    /// number of hard violations
    pub hard: u32,
    /// number of soft violations
    pub soft: u32,
}

impl Penalty {
    pub fn new() -> Self {
        Self { hard: 0, soft: 0 }
    }
}

impl Ord for Penalty {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hard.cmp(&other.hard).then(self.soft.cmp(&other.soft))
    }
}

impl PartialOrd for Penalty {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add for Penalty {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            hard: self.hard + rhs.hard,
            soft: self.soft + rhs.soft,
        }
    }
}

impl AddAssign for Penalty {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Display for Penalty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // when there are hard violations, the number of soft violations doesn't
        // matter, since the solution won't be accepted anyway
        if self.hard > 0 {
            write!(f, "hard violations: {}", self.hard)
        } else {
            write!(f, "total soft penalty: {}", self.soft)
        }
    }
}
