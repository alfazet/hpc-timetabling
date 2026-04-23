#ifndef GPU_TIMETABLING_SOLUTION_HPP
#define GPU_TIMETABLING_SOLUTION_HPP

#include <vector>

#include "student_assignment.hpp"
#include "model.hpp"
#include "penalty.hpp"

/// try to make it work without `config_preferences`
struct Solution {
    /// time slot assignments
    /// `times[i]` = assignment for the `i`-th class
    std::vector<TimeOption> times;
    /// room assignments
    /// `rooms[i]` = assignment for the `i`-th class,
    std::vector<RoomOption> rooms;
};

struct EvaluatedSolution {
    Solution inner;
    Penalty penalty;
    StudentAssignment student_assignment;

    EvaluatedSolution(Solution inner_, Penalty penalty_,
                      StudentAssignment student_assignment_);
};

#endif //GPU_TIMETABLING_SOLUTION_HPP