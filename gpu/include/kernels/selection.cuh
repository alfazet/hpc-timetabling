#ifndef GPU_TIMETABLING_SELECTION_CUH
#define GPU_TIMETABLING_SELECTION_CUH

#include "kernels/population.cuh"
#include "typedefs.hpp"

constexpr usize WARP_SIZE = 32;

namespace kernels {

struct Selection {
    // indices of selected solutions
    thrust::device_vector<u16> selected;
    f32 frac;

    Selection(usize population_size, f32 frac);

    void select(const Population &population);
};

} // namespace kernels

#endif // GPU_TIMETABLING_SELECTION_CUH
