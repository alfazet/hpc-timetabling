#ifndef GPU_TIMETABLING_MODEL_CUH
#define GPU_TIMETABLING_MODEL_CUH

#include "common.cuh"
#include "typedefs.hpp"
#include "parser/parser.hpp"

constexpr u32 NO_TRAVEL = UINT32_MAX;

namespace kernels {

struct RoomData {
    // the unavailabilities of room `i` begin at idx unavail_offsets[i]
    // and end at idx unavail_offsets[j] - 1
    // NOTE: this is an AoS, could be inefficient
    thrust::device_vector<parser::TimeSlots> unavail;
    thrust::device_vector<usize> unavail_offsets;
    // travel_time[i * n_rooms + j] = travel time between rooms `i` and `j`
    // USIZE_MAX if no travel is possible
    thrust::device_vector<u32> travel_time;
    thrust::device_vector<u32> capacity;
    usize n_rooms;

    RoomData(usize n_rooms, const std::vector<parser::TimeSlots> &unavail,
             const std::vector<usize> &unavail_offsets,
             const std::vector<u32> &travel_time,
             const std::vector<u32> &capacity);
};

// This struct should be allocated once on the GPU's heap.
// All the kernels should just take pointers to relevant parts of this struct
// to access the problem's data.
struct TimetableData {
    RoomData room_data;

    parser::Optimization optimization;
    u32 n_days;
    u32 n_weeks;
    u32 slots_per_day;

    TimetableData(u32 n_days, u32 n_weeks, u32 slots_per_day,
                  parser::Optimization optimization,
                  RoomData room_data);

    static TimetableData from_problem(parser::Problem p);
};
}

#endif // GPU_TIMETABLING_MODEL_CUH