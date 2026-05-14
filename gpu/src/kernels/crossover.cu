#include <curand_kernel.h>
#include <thrust/sequence.h>

#include "kernels/crossover.cuh"

namespace kernels {

Crossover::Crossover(f32 prob) : prob(prob) {}

// each subpart of every child is taken entirely either from one parent or the other
__global__ void k_subpart_crossover(u16 *new_config_prefs, u16 *new_times, u16 *new_rooms, const u16 *old_config_prefs,
                                    const u16 *old_times, const u16 *old_rooms, const u16 *selected, usize n_selected,
                                    usize n_classes, usize n_students, usize n_new, const u16 *class_subpart_idx,
                                    f32 prob, u32 seed) {
    usize tid = blockIdx.x * blockDim.x + threadIdx.x;
    if (tid >= n_new) {
        return;
    }

    curandState rng;
    curand_init(seed, tid, 0, &rng);
    u16 p1 = selected[curand(&rng) % n_selected];
    u16 p2 = selected[curand(&rng) % n_selected];

    bool do_crossover = curand_uniform(&rng) < prob;
    u32 child_rand = curand(&rng);

    usize dst_offset_classes = n_classes * tid;
    for (usize i = 0; i < n_classes; i++) {
        usize src_offset_classes;
        if (!do_crossover) {
            src_offset_classes = n_classes * p1;
        } else {
            // Knuth hashing, the magic constant is (2^32 * golden ratio) truncated to 32 bits,
            // makes it so that data for every class belonging to one subpart will be taken from the same parent
            u32 hash = (static_cast<u32>(class_subpart_idx[i]) * 2654435761u) ^ child_rand;
            u32 rem = hash & 1;
            src_offset_classes = n_classes * (rem * p1 + (1 - rem) * p2);
        }
        new_times[dst_offset_classes + i] = old_times[src_offset_classes + i];
        new_rooms[dst_offset_classes + i] = old_rooms[src_offset_classes + i];
    }

    usize dst_offset_students = n_students * MAX_COURSES_PER_STUDENT * tid;
    for (usize i = 0; i < n_students; i++) {
        usize src_offset_students;
        if (!do_crossover) {
            src_offset_students = n_students * MAX_COURSES_PER_STUDENT * p1;
        } else {
            u32 hash = (static_cast<u32>(class_subpart_idx[i]) * 2654435761u) ^ child_rand;
            u32 rem = hash & 1;
            src_offset_students = n_students * MAX_COURSES_PER_STUDENT * (rem * p1 + (1 - rem) * p2);
        }
        usize src_total_offset_students = src_offset_students + i * MAX_COURSES_PER_STUDENT;
        usize dst_total_offset_students = dst_offset_students + i * MAX_COURSES_PER_STUDENT;
        for (usize j = 0; j < MAX_COURSES_PER_STUDENT; j++) {
            new_config_prefs[dst_total_offset_students + j] = old_config_prefs[src_total_offset_students + j];
        }
    }
}

void Crossover::next_population(const Selection &selection, Population &population, const TimetableData &data) {
    usize n_classes = population.n_classes;
    usize n_students = population.n_students;
    usize pop_size = population.population_size;
    usize n_elites = std::ceil(population.population_size * population.elites_frac);
    usize n_new = pop_size - n_elites;
    usize n_selected = selection.selected.size();

    // copy the elite solutions without modification ...
    thrust::device_vector<u16> new_config_prefs(pop_size * n_students * MAX_COURSES_PER_STUDENT);
    thrust::device_vector<u16> new_times(pop_size * n_classes);
    thrust::device_vector<u16> new_rooms(pop_size * n_classes);
    const u16 *d_order = thrust::raw_pointer_cast(population.order.data());
    std::vector<u16> h_order(n_elites);
    cudaErrCheck(cudaMemcpy(h_order.data(), d_order, n_elites * sizeof(u16), cudaMemcpyDeviceToHost));
    const u16 *d_old_config_prefs = thrust::raw_pointer_cast(population.config_prefs.data());
    const u16 *d_old_times = thrust::raw_pointer_cast(population.times.data());
    const u16 *d_old_rooms = thrust::raw_pointer_cast(population.rooms.data());
    u16 *d_new_config_prefs = thrust::raw_pointer_cast(new_config_prefs.data());
    u16 *d_new_times = thrust::raw_pointer_cast(new_times.data());
    u16 *d_new_rooms = thrust::raw_pointer_cast(new_rooms.data());
    for (usize i = 0; i < n_elites; i++) {
        cudaErrCheck(cudaMemcpy(d_new_config_prefs + n_students * MAX_COURSES_PER_STUDENT * i,
                                d_old_config_prefs + n_students * MAX_COURSES_PER_STUDENT * h_order[i],
                                n_students * MAX_COURSES_PER_STUDENT * sizeof(u16), cudaMemcpyDeviceToDevice));
        cudaErrCheck(cudaMemcpy(d_new_times + n_classes * i, d_old_times + n_classes * h_order[i],
                                n_classes * sizeof(u16), cudaMemcpyDeviceToDevice));
        cudaErrCheck(cudaMemcpy(d_new_rooms + n_classes * i, d_old_rooms + n_classes * h_order[i],
                                n_classes * sizeof(u16), cudaMemcpyDeviceToDevice));
    }

    // ... and add new ones generated from the selection
    const u16 *d_selected = thrust::raw_pointer_cast(selection.selected.data());
    const u16 *d_subpart_idx = thrust::raw_pointer_cast(data.classes.subpart_idx.data());
    u32 seed = population.seed ^ static_cast<u32>(rand());
    constexpr u32 block_dim = SMALL_BLOCK_SIZE;
    u32 grid_dim = (n_new + block_dim - 1) / block_dim;
    k_subpart_crossover<<<grid_dim, block_dim>>>(
        d_new_config_prefs, d_new_times + n_elites * n_classes, d_new_rooms + n_elites * n_classes, d_old_config_prefs,
        d_old_times, d_old_rooms, d_selected, n_selected, n_classes, n_students, n_new, d_subpart_idx, prob, seed);
    cudaErrCheck(cudaDeviceSynchronize());
    population.times.swap(new_times);
    population.rooms.swap(new_rooms);
    thrust::sequence(population.order.begin(), population.order.end());
}

} // namespace kernels
