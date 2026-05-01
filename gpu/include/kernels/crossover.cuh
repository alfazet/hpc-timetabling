#ifndef GPU_TIMETABLING_CROSSOVER_CUH
#define GPU_TIMETABLING_CROSSOVER_CUH

#include "elitism.cuh"
#include "selection.cuh"
#include "typedefs.hpp"

namespace kernels {
struct Crossover {
    f32 prob;

    explicit Crossover(f32 prob);

    // replace the population with new solutions generated from the best ones + elites
    void next_population(const Selection &selection, Population &population, const Elitism &elitism);
};
} // namespace kernels

#endif // GPU_TIMETABLING_CROSSOVER_CUH
