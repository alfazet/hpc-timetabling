#ifndef GPU_TIMETABLING_SOLVER_CUH
#define GPU_TIMETABLING_SOLVER_CUH

#include "kernels/model.cuh"
#include "serializer/serializer.hpp"

struct FoundSolution {
    // student_assignment[i] = ids of students taking class with index `i`
    std::vector<std::vector<u16>> student_assignment;
    std::vector<u16> times_idxs; // idx of the time_option
    std::vector<u16> rooms_idxs; // idx of the room_option (not of the room itself!)
    kernels::Penalty penalty;

    FoundSolution(std::vector<std::vector<u16>> student_assignment, std::vector<u16> times_idxs,
                  std::vector<u16> rooms_idxs, kernels::Penalty penalty);

    serializer::Output serialize(const kernels::TimetableData &d_data) const;
};

struct Solver {
    kernels::TimetableData d_data;
    u32 generations;
    u32 population_size;
    f32 sel_frac;
    f32 cross_rate;
    f32 mut_rate;
    u32 mut_trials;
    f32 elites_frac;
    f32 worst_frac;
    u32 ls_iters;
    u32 seed;
    bool *stopper;

    Solver(kernels::TimetableData d_data, u32 generations, u32 population_size, f32 sel_frac, f32 cross_rate,
           f32 mut_rate, u32 mut_trials, f32 elites_frac, f32 worst_frac, u32 ls_iters, u32 seed, bool *stopper);

    void print_metadata() const;

    FoundSolution solve() const;
};

#endif // GPU_TIMETABLING_SOLVER_CUH
