#include <curand_kernel.h>

#include "kernels/mutation.cuh"
#include "kernels/utils.cuh"

namespace kernels {

Mutation::Mutation(f32 prob, u32 n_trials) : prob(prob), n_trials(n_trials) {}

__device__ static u32 count_violations(usize cls, u16 time_idx, u16 room_idx, const u16 *sh_times, const u16 *sh_rooms,
                                       usize n_classes, const parser::TimeSlots *time_opt_times,
                                       const u16 *room_opt_room_idx, const parser::TimeSlots *room_unavail,
                                       const usize *room_unavail_offsets, usize n_rooms, usize n_unavail) {
    u32 violations = 0;
    if (room_idx == NO_ROOM) {
        return 0;
    }
    u16 real_room = room_opt_room_idx[room_idx];
    const parser::TimeSlots &cls_time = time_opt_times[time_idx];
    for (usize c2 = 0; c2 < n_classes; c2++) {
        if (c2 == cls) {
            continue;
        }
        u16 c2_r = sh_rooms[c2];
        if (c2_r == NO_ROOM) {
            continue;
        }
        if (room_opt_room_idx[c2_r] == real_room) {
            if (utils::timeslots_overlap(cls_time, time_opt_times[sh_times[c2]])) {
                violations++;
            }
        }
    }
    usize unavail_start = room_unavail_offsets[real_room];
    usize unavail_end = real_room < n_rooms - 1 ? room_unavail_offsets[real_room + 1] : n_unavail;
    for (usize u = unavail_start; u < unavail_end; u++) {
        if (utils::timeslots_overlap(room_unavail[u], cls_time)) {
            violations++;
            break;
        }
    }

    return violations;
}

__global__ void k_mutations(u16 *pop_times, u16 *pop_rooms, usize n_classes, usize n_elites,
                            const u16 *class_times_start, const u16 *class_times_end, const u16 *class_rooms_start,
                            const u16 *class_rooms_end, const parser::TimeSlots *time_opt_times,
                            const u16 *room_opt_room_idx, const parser::TimeSlots *room_unavail,
                            const usize *room_unavail_offsets, usize n_rooms, usize n_unavail, f32 prob, u32 n_trials,
                            u32 seed) {
    usize tid = threadIdx.x;
    usize sol_offset = (n_elites + blockIdx.x) * n_classes;

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

    curandState rng;
    curand_init(seed, blockIdx.x * blockDim.x + tid, 0, &rng);

    // process in batches with __syncthreads inbetween so that later mutation attempts
    // are aware of what has already changed
    for (usize batch_start = 0; batch_start < n_classes; batch_start += blockDim.x) {
        usize cls = batch_start + tid;

        bool has_mutated = false;
        u16 final_t = 0;
        u16 final_r = 0;
        if (cls < n_classes) {
            u16 old_t = sh_times[cls];
            u16 old_r = sh_rooms[cls];

            u32 violations =
                count_violations(cls, old_t, old_r, sh_times, sh_rooms, n_classes, time_opt_times, room_opt_room_idx,
                                 room_unavail, room_unavail_offsets, n_rooms, n_unavail);
            // the adjustments that lead to violations have a higher chance to be mutated away,
            // and those with no violations have a higher chance to be kept intact
            f32 adjusted_prob = violations > 0 ? fminf(1.0f, prob * (1.0f + 2.0f * violations)) : prob * 0.25f;
            if (curand_uniform(&rng) <= adjusted_prob) {
                u16 t_start = class_times_start[cls];
                u16 t_end = class_times_end[cls];
                u16 n_times = t_end - t_start;
                u16 r_start = class_rooms_start[cls];
                u16 r_end = class_rooms_end[cls];
                bool needs_room = r_start != r_end;

                // try n_trials random alternatives and keep the best one
                u16 best_t = old_t;
                u16 best_r = old_r;
                u32 best_violations = violations;
                for (u32 trial = 0; trial < n_trials; trial++) {
                    u16 new_t = t_start + (n_times > 0 ? curand(&rng) % n_times : 0);
                    u16 new_r;
                    if (!needs_room) {
                        new_r = NO_ROOM;
                    } else {
                        new_r = r_start + curand(&rng) % (r_end - r_start);
                    }
                    u32 new_violations =
                        count_violations(cls, new_t, new_r, sh_times, sh_rooms, n_classes, time_opt_times,
                                         room_opt_room_idx, room_unavail, room_unavail_offsets, n_rooms, n_unavail);
                    if (new_violations < best_violations) {
                        best_violations = new_violations;
                        best_t = new_t;
                        best_r = new_r;
                    }
                }

                // fallback to random if nothing better was found
                if (best_t == old_t && best_r == old_r) {
                    best_t = t_start + (n_times > 0 ? curand(&rng) % n_times : 0);
                    if (!needs_room) {
                        best_r = NO_ROOM;
                    } else {
                        best_r = r_start + curand(&rng) % (r_end - r_start);
                    }
                }

                final_t = best_t;
                final_r = best_r;
                has_mutated = true;
            }
        }
        __syncthreads();

        if (cls < n_classes && has_mutated) {
            sh_times[cls] = final_t;
            sh_rooms[cls] = final_r;
            pop_times[sol_offset + cls] = final_t;
            pop_rooms[sol_offset + cls] = final_r;
        }
        __syncthreads();
    }
}

void Mutation::apply_mutations(Population &population, const TimetableData &data) {
    // skip the elites
    usize n_classes = population.n_classes;
    usize n_elites = std::ceil(population.population_size * population.elites_frac);
    u16 *pop_times = thrust::raw_pointer_cast(population.times.data());
    u16 *pop_rooms = thrust::raw_pointer_cast(population.rooms.data());
    const u16 *class_times_start = thrust::raw_pointer_cast(data.classes.times_start.data());
    const u16 *class_times_end = thrust::raw_pointer_cast(data.classes.times_end.data());
    const u16 *class_rooms_start = thrust::raw_pointer_cast(data.classes.rooms_start.data());
    const u16 *class_rooms_end = thrust::raw_pointer_cast(data.classes.rooms_end.data());

    const parser::TimeSlots *time_opt_times = thrust::raw_pointer_cast(data.time_options.times.data());
    const u16 *room_opt_room_idx = thrust::raw_pointer_cast(data.room_options.room_idx.data());
    const parser::TimeSlots *room_unavail = thrust::raw_pointer_cast(data.room_data.unavail.data());
    const usize *room_unavail_offsets = thrust::raw_pointer_cast(data.room_data.unavail_offsets.data());
    usize n_rooms = data.room_data.n_rooms;
    usize n_unavail = data.room_data.unavail.size();

    u32 seed = population.seed ^ static_cast<u32>(rand());
    constexpr u32 block_dim = BLOCK_SIZE;
    u32 grid_dim = static_cast<u32>(population.population_size - n_elites);

    usize sh_mem_size = 2 * n_classes * sizeof(u16);
    k_mutations<<<grid_dim, block_dim, sh_mem_size>>>(pop_times, pop_rooms, n_classes, n_elites, class_times_start,
                                                      class_times_end, class_rooms_start, class_rooms_end,
                                                      time_opt_times, room_opt_room_idx, room_unavail,
                                                      room_unavail_offsets, n_rooms, n_unavail, prob, n_trials, seed);
    cudaErrCheck(cudaDeviceSynchronize());
}

} // namespace kernels