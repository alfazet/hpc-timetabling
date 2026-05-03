#include <curand_kernel.h>
#include <thrust/sequence.h>

#include "kernels/crossover.cuh"

namespace kernels {

Crossover::Crossover(f32 prob) : prob(prob) {}

__global__ void k_one_point_crossover(u16 *new_times, u16 *new_rooms, const u16 *old_times, const u16 *old_rooms,
                                      const u16 *selected, usize n_selected, usize n_classes, usize n_new, f32 prob,
                                      u32 seed) {
    usize tid = blockIdx.x * blockDim.x + threadIdx.x;
    if (tid >= n_new) {
        return;
    }

    curandState rng;
    curand_init(seed, tid, 0, &rng);
    u16 p1 = selected[curand(&rng) % n_selected];
    u16 p2 = selected[curand(&rng) % n_selected];
    usize crossover_point = curand_uniform(&rng) < prob ? curand(&rng) % n_classes : n_classes;
    usize dst_offset = n_classes * tid;
    for (usize i = 0; i < n_classes; i++) {
        usize src_offset = i < crossover_point ? n_classes * p1 : n_classes * p2;
        new_times[dst_offset + i] = old_times[src_offset + i];
        new_rooms[dst_offset + i] = old_rooms[src_offset + i];
    }
}

void Crossover::next_population(const Selection &selection, Population &population) {
    usize n_classes = population.n_classes;
    usize pop_size = population.population_size;
    usize n_elites = population.n_elites;
    usize n_new = pop_size - n_elites;
    usize n_selected = selection.selected.size();

    // copy the elite solutions without modification ...
    thrust::device_vector<u16> new_times(pop_size * n_classes);
    thrust::device_vector<u16> new_rooms(pop_size * n_classes);
    const u16 *d_order = thrust::raw_pointer_cast(population.order.data());
    std::vector<u16> h_order(n_elites);
    cudaErrCheck(cudaMemcpy(h_order.data(), d_order, n_elites * sizeof(u16), cudaMemcpyDeviceToHost));
    const u16 *d_old_times = thrust::raw_pointer_cast(population.times.data());
    const u16 *d_old_rooms = thrust::raw_pointer_cast(population.rooms.data());
    u16 *d_new_times = thrust::raw_pointer_cast(new_times.data());
    u16 *d_new_rooms = thrust::raw_pointer_cast(new_rooms.data());
    for (usize i = 0; i < n_elites; i++) {
        cudaErrCheck(cudaMemcpy(d_new_times + n_classes * i, d_old_times + n_classes * h_order[i],
                                n_classes * sizeof(u16), cudaMemcpyDeviceToDevice));
        cudaErrCheck(cudaMemcpy(d_new_rooms + n_classes * i, d_old_rooms + n_classes * h_order[i],
                                n_classes * sizeof(u16), cudaMemcpyDeviceToDevice));
    }

    /// ... and add new ones generated from the selection
    const u16 *d_selected = thrust::raw_pointer_cast(selection.selected.data());
    u32 seed = population.seed ^ static_cast<u32>(rand());
    constexpr u32 block_dim = 1024;
    u32 grid_dim = (n_new + block_dim - 1) / block_dim;
    k_one_point_crossover<<<grid_dim, block_dim>>>(d_new_times + n_elites * n_classes,
                                                   d_new_rooms + n_elites * n_classes, d_old_times, d_old_rooms,
                                                   d_selected, n_selected, n_classes, n_new, prob, seed);
    cudaErrCheck(cudaDeviceSynchronize());
    population.times.swap(new_times);
    population.rooms.swap(new_rooms);
    thrust::sequence(population.order.begin(), population.order.end());
}

} // namespace kernels
