#ifndef GPU_TIMETABLING_POPULATION_CUH
#define GPU_TIMETABLING_POPULATION_CUH

#include "model.cuh"

namespace kernels {

// All data about all solutions is packed into a flat array of
// size n_classes * population_size.
// For example: times[i * n_classes + j]
// represents the time slot assignment for the `j`-th class in the
// `i`-th solution
struct Population {
    // time slot assignments
    // `times[i]` = assignment for the `i`-th class
    thrust::device_vector<TimeOption> times;
    // room assignments
    // `rooms[i]` = assignment for the `i`-th class,
    thrust::device_vector<RoomOption> rooms;
    usize n_classes;
};

}

#endif //GPU_TIMETABLING_POPULATION_CUH