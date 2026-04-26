#ifndef GPU_TIMETABLING_SOLVER_CUH
#define GPU_TIMETABLING_SOLVER_CUH

#include "kernels/model.cuh"
#include "serializer/serializer.hpp"

struct FoundSolution {
    // student_assignment[i] = indices of students taking class with index `i`
    std::vector<std::vector<usize> > student_assignment;
    std::vector<parser::TimeSlots> times;
    std::vector<usize> rooms_idxs;
    std::pair<u32, u32> penalty; // {hard, soft}

    FoundSolution(std::vector<std::vector<usize> > student_assignment,
                  std::vector<parser::TimeSlots> times,
                  std::vector<usize> rooms_idxs,
                  std::pair<u32, u32> penalty);

    serializer::Output serialize(const std::vector<parser::RoomId> &room_ids,
                                 const std::vector<parser::StudentId> &
                                 student_ids,
                                 const std::vector<parser::ClassId> &class_ids)
    const;
};

struct Solver {
    u32 generations;
    u32 population_size;
    kernels::TimetableData d_data;

    Solver(u32 generations, u32 population_size, kernels::TimetableData d_data);
};

#endif //GPU_TIMETABLING_SOLVER_CUH