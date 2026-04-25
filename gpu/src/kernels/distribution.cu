#include "kernels/distribution.cuh"

namespace kernels::distribution {

__device__ void calculate_penalty(const Population *pop,
                                  const TimetableData *d_data, u32 *penalty) {
    // each thread that calls this function should compute the penalty of its
    // solution - so it should only access its own part of the `pop` array
}
}