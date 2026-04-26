#ifndef GPU_TIMETABLING_SOLVER_CUH
#define GPU_TIMETABLING_SOLVER_CUH

#include "kernels/kernels.cuh"
#include "serializer/serializer.hpp"

struct FoundSolution {
    std::vector<std::vector<usize> > student_assignment;
    std::vector<usize> times;
    std::vector<usize> rooms;
    std::pair<u32, u32> penalty; // {hard, soft}

    // FoundSolution(std::vector<std::vector<usize> > student_assignment,
    //               std::vector<kernels::TimeOption> times,
    //               std::vector<kernels::RoomOption> rooms,
    //               std::pair<u32, u32> penalty);
};

struct Solver {
    u32 generations;
    u32 population_size;
    kernels::TimetableData d_data;

    Solver(u32 generations, u32 population_size, kernels::TimetableData d_data);
};

#endif //GPU_TIMETABLING_SOLVER_CUH