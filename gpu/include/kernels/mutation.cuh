#ifndef GPU_TIMETABLING_MUTATION_CUH
#define GPU_TIMETABLING_MUTATION_CUH

#include "population.cuh"
#include "typedefs.hpp"

namespace kernels {

struct Mutation {
    f32 prob;

    explicit Mutation(f32 prob);

    // apply mutations - only to the part of the population
    // that was created by crossing-over (not the elites
    void apply_mutations(Population &population, const TimetableData &data);
};

} // namespace kernels

#endif // GPU_TIMETABLING_MUTATION_CUH
