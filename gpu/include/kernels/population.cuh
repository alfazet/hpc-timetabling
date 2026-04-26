#ifndef GPU_TIMETABLING_POPULATION_CUH
#define GPU_TIMETABLING_POPULATION_CUH

#include "model.cuh"

namespace kernels {

// All data about all solutions is packed into a flat array of
// size n_classes * population_size.
// For example: times[i * n_classes + j]
// represents the index of the time option chosen for the `j`-th class in the
// `i`-th solution.
// Indices refer to the TimetableData::time_options/room_options vectors.
struct Population {
    // time slot assignments
    // `times[i]` = idx of the TimeOption chosen for the `i`-th class
    thrust::device_vector<usize> times;
    // room assignments
    // `times[i]` = idx of the RoomOption chosen for the `i`-th class
    // NO_ROOM if the class doesn't need a room
    thrust::device_vector<usize> rooms;
    u32 seed;
    usize n_classes;
    usize population_size;

    Population(usize n_classes, usize population_size, u64 seed);

    // initialize the population with random solutions
    void init(const TimetableData &d_data);
};

}

#endif // GPU_TIMETABLING_POPULATION_CUH