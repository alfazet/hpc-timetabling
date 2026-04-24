#ifndef GPU_TIMETABLING_DISTRIBUTION_CUH
#define GPU_TIMETABLING_DISTRIBUTION_CUH

#include "solution.cuh"
#include "common.cuh"

struct Distribution {
    // Assume that sol and data are pointers to structs in gpu memory
    const Solution *sol;
    const TimetableData *data;

    Distribution(const Solution *, const TimetableData *);
    u32 calculate_penalty() const;
};

namespace kernels {
    __global__ void distribution_penalty_calculation(const Solution *,
        const TimetableData *, u32 *, const usize);
}

#endif // GPU_TIMETABLING_DISTRIBUTION_CUH
