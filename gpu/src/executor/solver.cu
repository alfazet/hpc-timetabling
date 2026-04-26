#include "executor/solver.cuh"
#include "kernels/model.cuh"

// FoundSolution::FoundSolution(
//     std::vector<std::vector<usize> > student_assignment,
//     std::vector<kernels::TimeOption> times,
//     std::vector<kernels::RoomOption> rooms,
//     std::pair<u32, u32> penalty)
//     : student_assignment(std::move(student_assignment)),
//       times(std::move(times)), rooms(std::move(rooms)),
//       penalty(std::move(penalty)) {
// }

Solver::Solver(u32 generations, u32 population_size,
               kernels::TimetableData d_data)
    : generations(generations), population_size(population_size),
      d_data(d_data) {
}