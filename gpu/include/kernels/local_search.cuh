#ifndef GPU_TIMETABLING_LOCAL_SEARCH_CUH
#define GPU_TIMETABLING_LOCAL_SEARCH_CUH

#include "population.cuh"
#include "typedefs.hpp"

namespace kernels {

struct LocalSearch {
    u32 n_iters;
    u32 n_trials;

    LocalSearch(u32 n_iters, u32 n_trials);

    // apply local search to all solutions in the population
    // modifies times/rooms in-place (one block per one solution)
    void search(Population &population, const TimetableData &data);
};

} // namespace kernels

#endif // GPU_TIMETABLING_LOCAL_SEARCH_CUH
