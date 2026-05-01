#ifndef GPU_TIMETABLING_ELITISM_CUH
#define GPU_TIMETABLING_ELITISM_CUH

#include "kernels/population.cuh"
#include "typedefs.hpp"

namespace kernels {

struct Elitism {
    // indices of elite solution that shouldn't be considered for crossover/mutations
    thrust::device_vector<u16> elites;
    f32 frac;

    Elitism(usize population_size, f32 frac);

    void choose_elites(const Population &population);
};

} // namespace kernels

#endif // GPU_TIMETABLING_ELITISM_CUH
