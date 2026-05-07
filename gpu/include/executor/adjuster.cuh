#ifndef GPU_TIMETABLING_ADJUSTER_CUH
#define GPU_TIMETABLING_ADJUSTER_CUH

#include "kernels/crossover.cuh"
#include "kernels/mutation.cuh"
#include "kernels/penalty.cuh"
#include "typedefs.hpp"

struct Stats {
    usize generation = 0;
    usize stagnation = 0;
    usize progress = 0;
    kernels::Penalty min_penalty = kernels::MAX_PENALTY;

    void update(usize cur_generation, kernels::Penalty cur_penalty);

    void print(f32 mut_rate, f32 cross_rate) const;
};

struct Adjuster {
    f32 delta;
    f32 min_mut;
    f32 max_mut;
    f32 min_cross;
    f32 max_cross;

    Adjuster(f32 delta, f32 min_mut, f32 max_mut, f32 min_cross, f32 max_cross);

    void adjust(const Stats &stats, kernels::Mutation &mut, kernels::Crossover &cross) const;
};

#endif // GPU_TIMETABLING_ADJUSTER_CUH
