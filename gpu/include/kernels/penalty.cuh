#ifndef GPU_TIMETABLING_PENALTY_CUH
#define GPU_TIMETABLING_PENALTY_CUH

#include <tuple>

#include "typedefs.hpp"

namespace kernels {

struct Penalty {
    u32 hard = 0;
    u32 soft = 0;

    __host__ __device__ Penalty() = default;

    __host__ __device__ Penalty(u32 hard, u32 soft) : hard(hard), soft(soft) {}

    bool operator <(const Penalty& p) const {
        if (hard == p.hard) {
            return soft < p.soft;
        }
        return hard < p.hard;
    }
};

}

#endif // GPU_TIMETABLING_PENALTY_CUH