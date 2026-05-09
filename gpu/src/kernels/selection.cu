#include <curand_kernel.h>

#include "kernels/selection.cuh"

namespace kernels {

Selection::Selection(usize population_size, f32 frac) : selected(std::ceil(population_size * frac)), frac(frac) {}

__global__ void k_tournament_select(u16 *selected, const Penalty *penalty, const u16 *order, usize n_selected,
                                    usize population_size, u32 seed) {
    usize warp_id = blockIdx.x * blockDim.x / WARP_SIZE + threadIdx.x / WARP_SIZE;
    if (warp_id >= n_selected) {
        return;
    }
    usize lane = threadIdx.x % WARP_SIZE;

    curandState rng;
    curand_init(seed, warp_id * WARP_SIZE + lane, 0, &rng);
    u16 winner_idx = curand(&rng) % population_size;
    u32 full_mask = __activemask();

    // each of the 32 lanes selects a random solution
    // the one with the least penalty becomes "selected"
    // penalties are sorted here, so we can just compare indices
    for (u32 offset = WARP_SIZE / 2; offset > 0; offset /= 2) {
        // __shfl_down_sync(..., var, offset) copies var from lane (x + offset) to lane x
        u16 other_idx = __shfl_down_sync(full_mask, winner_idx, offset);
        winner_idx = min(other_idx, winner_idx);
    }
    if (lane == 0) {
        selected[warp_id] = order[winner_idx];
    }
}

void Selection::select(const Population &population) {
    u32 n_selected = this->selected.size();
    u16 *d_selected = thrust::raw_pointer_cast(this->selected.data());
    const u16 *d_order = thrust::raw_pointer_cast(population.order.data());
    const Penalty *d_penalty = thrust::raw_pointer_cast(population.penalty.data());
    u32 seed = population.seed ^ static_cast<u32>(rand());

    // one warp selects one winner (so we need n_selected warps)
    constexpr usize WARPS_PER_BLOCK = BLOCK_SIZE / WARP_SIZE;
    constexpr dim3 block_dim(BLOCK_SIZE);
    dim3 grid_dim((n_selected + WARPS_PER_BLOCK - 1) / WARPS_PER_BLOCK);
    k_tournament_select<<<grid_dim, block_dim>>>(d_selected, d_penalty, d_order, n_selected, population.population_size,
                                                 seed);

    cudaErrCheck(cudaDeviceSynchronize());
}

} // namespace kernels
