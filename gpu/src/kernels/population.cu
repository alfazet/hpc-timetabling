#include "kernels/population.cuh"
#include <curand_kernel.h>

namespace kernels {

__global__ void k_init_population(usize *times,
                                  usize *rooms,
                                  usize n_classes, usize population_size,
                                  const usize *times_start,
                                  const usize *times_end,
                                  const usize *rooms_start,
                                  const usize *rooms_end,
                                  u32 seed) {
    usize sol = blockIdx.x * blockDim.x + threadIdx.x;
    usize cls = blockIdx.y * blockDim.y + threadIdx.y;
    if (sol >= population_size || cls >= n_classes) {
        return;
    }

    curandState rng;
    usize tid = sol * n_classes + cls;
    curand_init(seed, tid, 0, &rng);

    usize t_start = times_start[cls];
    usize t_end = times_end[cls];
    usize n_times = t_end - t_start;
    times[tid] = t_start + (n_times > 0 ? curand(&rng) % n_times : 0);

    usize r_start = rooms_start[cls];
    usize r_end = rooms_end[cls];
    if (r_start == r_end) {
        rooms[tid] = NO_ROOM;
    } else {
        usize n_rooms = r_end - r_start;
        rooms[tid] = r_start + curand(&rng) % n_rooms;
    }
}

Population::Population(usize n_classes, usize population_size, u64 seed)
    : times(thrust::device_vector<usize>(n_classes * population_size)),
      rooms(thrust::device_vector<usize>(n_classes * population_size)),
      seed(seed), n_classes(n_classes), population_size(population_size) {
}

void Population::init(const TimetableData &d_data) {
    const usize *d_times_start =
        thrust::raw_pointer_cast(d_data.classes.times_start.data());
    const usize *d_times_end =
        thrust::raw_pointer_cast(d_data.classes.times_end.data());
    const usize *d_rooms_start =
        thrust::raw_pointer_cast(d_data.classes.rooms_start.data());
    const usize *d_rooms_end =
        thrust::raw_pointer_cast(d_data.classes.rooms_end.data());

    usize *d_times = thrust::raw_pointer_cast(this->times.data());
    usize *d_rooms = thrust::raw_pointer_cast(this->rooms.data());

    // x: solutions, y: classes
    constexpr dim3 block_dim(32, 32); // arbitrary numbers for now
    const dim3 grid_dim(
        (static_cast<u32>(population_size) + block_dim.x - 1) / block_dim.x,
        (static_cast<u32>(n_classes) + block_dim.y - 1) / block_dim.y);
    k_init_population<<<grid_dim, block_dim>>>(
        d_times, d_rooms, n_classes, population_size, d_times_start,
        d_times_end,
        d_rooms_start, d_rooms_end, seed);

    cudaErrCheck(cudaDeviceSynchronize());
}

}