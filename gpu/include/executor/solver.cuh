#ifndef GPU_TIMETABLING_SOLVER_CUH
#define GPU_TIMETABLING_SOLVER_CUH

#include "kernels/model.cuh"
#include "serializer/serializer.hpp"

struct FoundSolution {
    // student_assignment[i] = indices of students taking class with index `i`
    std::vector<std::vector<u16> > student_assignment;
    std::vector<u16> times_idxs;
    std::vector<u16> rooms_idxs;
    std::pair<u32, u32> penalty; // {hard, soft}

    FoundSolution(std::vector<std::vector<u16> > student_assignment,
                  std::vector<u16> times_idxs,
                  std::vector<u16> rooms_idxs,
                  std::pair<u32, u32> penalty);

    serializer::Output serialize(const std::vector<parser::RoomId> &room_ids,
                                 const std::vector<parser::StudentId> &
                                 student_ids,
                                 const std::vector<parser::ClassId> &class_ids,
                                 const std::vector<parser::TimeSlots> &time_slots)
    const;
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