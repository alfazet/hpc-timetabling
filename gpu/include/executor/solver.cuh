#ifndef GPU_TIMETABLING_SOLVER_CUH
#define GPU_TIMETABLING_SOLVER_CUH

#include "kernels/model.cuh"
#include "serializer/serializer.hpp"

struct FoundSolution {
    // student_assignment[i] = ids of students taking class with index `i`
    std::vector<std::vector<u16> > student_assignment;
    std::vector<u16> times_idxs; // idx of the time_option
    std::vector<u16> rooms_idxs; // idx of the room_option (not of the room itself!)
    std::pair<u32, u32> penalty; // {hard, soft}

    FoundSolution(std::vector<std::vector<u16> > student_assignment,
                  std::vector<u16> times_idxs,
                  std::vector<u16> rooms_idxs,
                  std::pair<u32, u32> penalty);

    serializer::Output serialize(const kernels::TimetableData& d_data) const;
};

struct Solver {
    kernels::TimetableData d_data;
    u32 generations;
    u32 population_size;
    u32 seed;

    Solver(kernels::TimetableData d_data, u32 generations, u32 population_size,
           u32 seed);

    FoundSolution solve() const;
};

#endif //GPU_TIMETABLING_SOLVER_CUH