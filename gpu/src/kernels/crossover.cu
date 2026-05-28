#include <curand_kernel.h>
#include <thrust/sequence.h>

#include "kernels/crossover.cuh"
#include "kernels/utils.cuh"

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
            u32 hash = (static_cast<u32>(i) * 2654435761u) ^ child_rand;
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

__global__ void k_fix_room_conflicts(const u16 *pop_times, u16 *pop_rooms, usize n_classes,
                                     const u16 *class_rooms_start, const u16 *class_rooms_end,
                                     const parser::TimeSlots *time_opt_times, const u16 *room_opt_room_idx,
                                     const parser::TimeSlots *room_unavail, const usize *room_unavail_offsets,
                                     usize n_rooms, usize n_unavail) {
    usize tid = threadIdx.x;
    usize sol_offset = blockIdx.x * n_classes;

    extern __shared__ u8 sh_mem[];
    auto *sh_times = reinterpret_cast<u16 *>(sh_mem);
    // sh_times: n_classes * sizeof(u16)
    u16 *sh_rooms = sh_times + n_classes;
    // sh_rooms: n_classes * sizeof(u16)

    for (usize i = tid; i < n_classes; i += blockDim.x) {
        sh_times[i] = pop_times[sol_offset + i];
        sh_rooms[i] = pop_rooms[sol_offset + i];
    }
    __syncthreads();

    for (usize batch_start = 0; batch_start < n_classes; batch_start += blockDim.x) {
        usize cls = batch_start + tid;
        bool fix_found = false;
        u16 fixed_room = 0;

        if (cls < n_classes) {
            u16 old_room = sh_rooms[cls];
            u16 rooms_start = class_rooms_start[cls];
            u16 rooms_end = class_rooms_end[cls];

            if (old_room != NO_ROOM && rooms_start != rooms_end) {
                u16 old_real_room = room_opt_room_idx[old_room];
                const parser::TimeSlots &cls_time = time_opt_times[sh_times[cls]];

                u32 violations = 0;
                for (usize c2 = 0; c2 < n_classes; c2++) {
                    if (c2 == cls) {
                        continue;
                    }
                    u16 c2_r = sh_rooms[c2];
                    if (c2_r != NO_ROOM && room_opt_room_idx[c2_r] == old_real_room &&
                        utils::timeslots_overlap(cls_time, time_opt_times[sh_times[c2]])) {
                        violations++;
                    }
                }
                usize ua_start = room_unavail_offsets[old_real_room];
                usize ua_end = old_real_room < n_rooms - 1 ? room_unavail_offsets[old_real_room + 1] : n_unavail;
                for (usize u = ua_start; u < ua_end; u++) {
                    if (utils::timeslots_overlap(room_unavail[u], cls_time)) {
                        violations++;
                        break;
                    }
                }

                if (violations > 0) {
                    u16 best_r = old_room;
                    u32 best_violations = violations;

                    for (u16 r = rooms_start; r < rooms_end; r++) {
                        if (r == old_room) {
                            continue;
                        }
                        u16 real_room = room_opt_room_idx[r];

                        u32 cur_violations = 0;
                        for (usize c2 = 0; c2 < n_classes; c2++) {
                            if (c2 == cls) {
                                continue;
                            }
                            u16 c2_r = sh_rooms[c2];
                            if (c2_r != NO_ROOM && room_opt_room_idx[c2_r] == real_room &&
                                utils::timeslots_overlap(cls_time, time_opt_times[sh_times[c2]])) {
                                cur_violations++;
                            }
                        }
                        usize s = room_unavail_offsets[real_room];
                        usize e = real_room < n_rooms - 1 ? room_unavail_offsets[real_room + 1] : n_unavail;
                        for (usize u = s; u < e; u++) {
                            if (utils::timeslots_overlap(room_unavail[u], cls_time)) {
                                cur_violations++;
                                break;
                            }
                        }
                        if (cur_violations < best_violations) {
                            best_violations = cur_violations;
                            best_r = r;
                        }
                    }
                    if (best_r != old_room) {
                        fixed_room = best_r;
                        fix_found = true;
                    }
                }
            }
        }
        __syncthreads();

        if (cls < n_classes && fix_found) {
            sh_rooms[cls] = fixed_room;
            pop_rooms[sol_offset + cls] = fixed_room;
        }
        __syncthreads();
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

    // try to fix room conflicts in new solutions
    {
        const parser::TimeSlots *d_time_opt_times = thrust::raw_pointer_cast(data.time_options.times.data());
        const u16 *d_room_opt_room_idx = thrust::raw_pointer_cast(data.room_options.room_idx.data());
        const u16 *d_class_rooms_start = thrust::raw_pointer_cast(data.classes.rooms_start.data());
        const u16 *d_class_rooms_end = thrust::raw_pointer_cast(data.classes.rooms_end.data());
        const parser::TimeSlots *d_room_unavail = thrust::raw_pointer_cast(data.room_data.unavail.data());
        const usize *d_room_unavail_offsets = thrust::raw_pointer_cast(data.room_data.unavail_offsets.data());
        usize n_rooms = data.room_data.n_rooms;
        usize n_unavail = data.room_data.unavail.size();

        constexpr u32 block_dim = LARGE_BLOCK_SIZE;
        usize sh_mem_size = 2 * n_classes * sizeof(u16);
        k_fix_room_conflicts<<<static_cast<u32>(n_new), block_dim, sh_mem_size>>>(
            d_new_times + n_elites * n_classes, d_new_rooms + n_elites * n_classes, n_classes, d_class_rooms_start,
            d_class_rooms_end, d_time_opt_times, d_room_opt_room_idx, d_room_unavail, d_room_unavail_offsets, n_rooms,
            n_unavail);
        cudaErrCheck(cudaDeviceSynchronize());
    }
    population.times.swap(new_times);
    population.rooms.swap(new_rooms);
    thrust::sequence(population.order.begin(), population.order.end());
}

} // namespace kernels
