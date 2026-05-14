#ifndef GPU_TIMETABLING_POPULATION_CUH
#define GPU_TIMETABLING_POPULATION_CUH

#include "assigner.cuh"
#include "executor/solver.cuh"
#include "model.cuh"

constexpr usize MAX_COURSES_PER_STUDENT = 8;

namespace kernels {

struct StudentAssignment;

// All data about all solutions is packed into a flat array of
// size n_classes * population_size.
// For example: times[i * n_classes + j]
// represents the index of the time option chosen for the `j`-th class in the
// `i`-th solution.
// Indices refer to the TimetableData::time_options/room_options vectors.
struct Population {
    // indices of the preferred configs for each of the courses that each student wants to take
    // the index of the preferred config for the `k`-th course of the `j`-th student in the `i`-th solution
    // is placed at config_prefs[i * n_students * MAX_COURSES_PER_STUDENT + j * MAX_COURSES_PER_STUDENT + k]
    thrust::device_vector<u16> config_prefs;
    // time slot assignments
    thrust::device_vector<u16> times;
    // room assignments
    // NO_ROOM if the class doesn't need a room
    thrust::device_vector<u16> rooms;
    thrust::device_vector<Penalty> penalty;
    // indices of solution sorted by increasing penalty
    thrust::device_vector<u16> order;
    u32 seed;
    usize n_students;
    usize n_classes;
    usize population_size;
    f32 elites_frac;
    f32 worst_frac;

    Population(usize n_students, usize n_classes, usize population_size, f32 elites_frac, f32 worst_frac, u32 seed);

    // initialize the population with random solutions
    // (one thread per one solution)
    void init(const TimetableData &d_data);

    // sort by penalty
    void sort();

    Penalty get_best_penalty() const;

    // copy the solution with the least penalty to the host
    FoundSolution get_best_solution(const StudentAssignment &assignment) const;
};

} // namespace kernels

#endif // GPU_TIMETABLING_POPULATION_CUH