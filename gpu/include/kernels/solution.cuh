#ifndef GPU_TIMETABLING_SOLUTION_HPP
#define GPU_TIMETABLING_SOLUTION_HPP

#include "model.cuh"

/// try to make it work without `config_preferences`
struct Solution {
    thrust::device_vector<TimeOption> times_;
    /// time slot assignments
    /// `times[i]` = assignment for the `i`-th class
    TimeOption *times;
    usize times_count;

    thrust::device_vector<RoomOption> rooms_;
    /// room assignments
    /// `rooms[i]` = assignment for the `i`-th class,
    RoomOption *rooms;
    usize rooms_count;
};

struct EvaluatedSolution {
    Solution inner;
    u32 penalty;
    // StudentAssignment student_assignment;

    // EvaluatedSolution(Solution inner_, u32 penalty_,
    //                   StudentAssignment student_assignment_);
};

#endif //GPU_TIMETABLING_SOLUTION_HPP