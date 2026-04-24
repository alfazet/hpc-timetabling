#ifndef GPU_TIMETABLING_DISTRIBUTION_CUH
#define GPU_TIMETABLING_DISTRIBUTION_CUH

#include "executor/solution.hpp"
#include "common.cuh"

struct Distribution {
    const Solution &sol;
    const TimetableData &data;

    Distribution(const Solution &, const TimetableData &);
    Penalty calculate_penalty();
};

namespace kernels {
    __global__ void distribution_penalty_calculation(const Distribution *, u32 *);
}

#endif // GPU_TIMETABLING_DISTRIBUTION_CUH
