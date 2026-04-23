#ifndef GPU_TIMETABLING_SOLVER_HPP
#define GPU_TIMETABLING_SOLVER_HPP

#include "model.hpp"
#include "typedefs.hpp"
#include "solution.hpp"

struct Solver {
    u32 generations;
    u32 population_size;
    TimetableData data;

    Solver(u32 generations_, u32 population_size_, TimetableData data_);

    EvaluatedSolution solve();
};

#endif //GPU_TIMETABLING_SOLVER_HPP