#include <curand_kernel.h>

#include "kernels/mutation.cuh"

namespace kernels {

Mutation::Mutation(f32 prob) : prob(prob) {}

__global__ void k_mutations(u16 *pop_times, u16 *pop_rooms, usize n_classes, usize n_elites,
                            const u16 *class_times_start, const u16 *class_times_end, const u16 *class_rooms_start,
                            const u16 *class_rooms_end, f32 prob, u32 seed) {
    usize tid = blockIdx.x * blockDim.x + threadIdx.x;
    usize sol_offset = (n_elites + blockIdx.x) * n_classes;
    curandState rng;
    curand_init(seed, tid, 0, &rng);

    for (usize cls = threadIdx.x; cls < n_classes; cls += blockDim.x) {
        if (curand_uniform(&rng) > prob) {
            continue;
        }
        u16 t_start = class_times_start[cls];
        u16 t_end = class_times_end[cls];
        u16 n_times = t_end - t_start;
        pop_times[sol_offset + cls] = t_start + (n_times > 0 ? curand(&rng) % n_times : 0);
        u16 r_start = class_rooms_start[cls];
        u16 r_end = class_rooms_end[cls];
        if (r_start == r_end) {
            pop_rooms[sol_offset + cls] = NO_ROOM;
        } else {
            pop_rooms[sol_offset + cls] = r_start + curand(&rng) % (r_end - r_start);
        }
    }
}

void Mutation::apply_mutations(Population &population, const TimetableData &data) {
    // skip the elites
    usize n_classes = population.n_classes;
    usize n_elites = population.n_elites;
    u16 *pop_times = thrust::raw_pointer_cast(population.times.data());
    u16 *pop_rooms = thrust::raw_pointer_cast(population.rooms.data());
    const u16 *class_times_start = thrust::raw_pointer_cast(data.classes.times_start.data());
    const u16 *class_times_end = thrust::raw_pointer_cast(data.classes.times_end.data());
    const u16 *class_rooms_start = thrust::raw_pointer_cast(data.classes.rooms_start.data());
    const u16 *class_rooms_end = thrust::raw_pointer_cast(data.classes.rooms_end.data());

    u32 seed = population.seed ^ static_cast<u32>(rand());
    constexpr u32 block_dim = 1024;
    u32 grid_dim = static_cast<u32>(population.population_size - n_elites);
    k_mutations<<<grid_dim, block_dim>>>(pop_times, pop_rooms, n_classes, n_elites, class_times_start, class_times_end,
                                         class_rooms_start, class_rooms_end, prob, seed);
    cudaErrCheck(cudaDeviceSynchronize());
}

} // namespace kernels