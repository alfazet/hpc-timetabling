#ifndef GPU_TIMETABLING_DISTRIBUTION_CUH
#define GPU_TIMETABLING_DISTRIBUTION_CUH

#include "population.cuh"

namespace kernels::distribution {

__device__ void calculate_penalty(const TimetableData *d_data, u32 *penalty);
}

#endif //GPU_TIMETABLING_DISTRIBUTION_CUH