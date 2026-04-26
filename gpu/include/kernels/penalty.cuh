#ifndef GPU_TIMETABLING_PENALTY_CUH
#define GPU_TIMETABLING_PENALTY_CUH

#include "typedefs.hpp"
#include "common.cuh"

namespace kernels {

struct Penalty {
    uint2 penalty;

    explicit Penalty(u32 hard, u32 soft);
};

}

#endif //GPU_TIMETABLING_PENALTY_CUH