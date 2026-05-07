#include <curand_kernel.h>

#include "kernels/local_search.cuh"
#include "kernels/utils.cuh"

constexpr int2 NEUTRAL{0, 0};

namespace kernels {

LocalSearch::LocalSearch(u32 n_iters, u32 n_trials) : n_iters(n_iters), n_trials(n_trials) {}

__device__ inline bool cmp_delta(int2 a, int2 b) {
    if (a.y == b.y) {
        return a.x < b.x;
    }
    return a.y < b.y;
}

// checks how the move affects time/room conflicts
// returns (hard_delta, soft_delta), negative values are improvements
__device__ static int2 compute_move_delta(usize cls, u16 old_t, u16 old_r, u16 new_t, u16 new_r, const u16 *sh_times,
                                          const u16 *sh_rooms, usize n_classes, const parser::TimeSlots *time_opt_times,
                                          const u32 *time_opt_penalty, const u16 *room_opt_room_idx,
                                          const u32 *room_opt_penalty, const parser::TimeSlots *room_unavail,
                                          const usize *room_unavail_offsets, usize n_rooms, usize n_unavail,
                                          u32 opt_time, u32 opt_room) {
    i32 delta_hard = 0;
    i32 delta_soft = 0;

    delta_soft += time_opt_penalty[new_t] - time_opt_penalty[old_t];
    i32 old_room_pen = old_r != NO_ROOM ? room_opt_penalty[old_r] : 0;
    i32 new_room_pen = new_r != NO_ROOM ? room_opt_penalty[new_r] : 0;
    delta_soft += new_room_pen - old_room_pen;
    delta_soft =
        (time_opt_penalty[new_t] - time_opt_penalty[old_t]) * opt_time + (new_room_pen - old_room_pen) * opt_room;

    u16 old_real_room = old_r != NO_ROOM ? room_opt_room_idx[old_r] : NO_ROOM;
    u16 new_real_room = new_r != NO_ROOM ? room_opt_room_idx[new_r] : NO_ROOM;

    const parser::TimeSlots &old_time = time_opt_times[old_t];
    const parser::TimeSlots &new_time = time_opt_times[new_t];
    if (old_real_room != NO_ROOM) {
        usize unavail_start = room_unavail_offsets[old_real_room];
        usize unavail_end = old_real_room < n_rooms - 1 ? room_unavail_offsets[old_real_room + 1] : n_unavail;
        for (usize u = unavail_start; u < unavail_end; u++) {
            if (utils::timeslots_overlap(room_unavail[u], old_time)) {
                delta_hard--;
                break;
            }
        }
    }
    if (new_real_room != NO_ROOM) {
        usize unavail_start = room_unavail_offsets[new_real_room];
        usize unavail_end = new_real_room < n_rooms - 1 ? room_unavail_offsets[new_real_room + 1] : n_unavail;
        for (usize u = unavail_start; u < unavail_end; u++) {
            if (utils::timeslots_overlap(room_unavail[u], new_time)) {
                delta_hard++;
                break;
            }
        }
    }

    for (usize c2 = 0; c2 < n_classes; c2++) {
        if (c2 == cls) {
            continue;
        }
        u16 c2_r = sh_rooms[c2];
        if (c2_r == NO_ROOM) {
            continue;
        }
        u16 c2_real_room = room_opt_room_idx[c2_r];
        const parser::TimeSlots &c2_time = time_opt_times[sh_times[c2]];

        if (old_real_room != NO_ROOM && old_real_room == c2_real_room && utils::timeslots_overlap(old_time, c2_time)) {
            delta_hard--;
        }
        if (new_real_room != NO_ROOM && new_real_room == c2_real_room && utils::timeslots_overlap(new_time, c2_time)) {
            delta_hard++;
        }
    }

    return make_int2(delta_hard, delta_soft);
}

__global__ void k_local_search(u16 *pop_times, u16 *pop_rooms, usize n_classes, const u16 *class_times_start,
                               const u16 *class_times_end, const u16 *class_rooms_start, const u16 *class_rooms_end,
                               const parser::TimeSlots *time_opt_times, const u32 *time_opt_penalty,
                               const u16 *room_opt_room_idx, const u32 *room_opt_penalty,
                               const parser::TimeSlots *room_unavail, const usize *room_unavail_offsets, usize n_rooms,
                               usize n_unavail, u32 opt_time, u32 opt_room, u32 n_iterations, u32 n_trials, u32 seed) {
    usize sol = blockIdx.x;
    usize tid = threadIdx.x;
    usize block_size = blockDim.x;
    usize sol_offset = sol * n_classes;

    extern __shared__ u8 sh_mem[];
    auto *sh_delta = reinterpret_cast<int2 *>(sh_mem);
    // sh_delta: block_size * sizeof(int2)
    auto *sh_times = reinterpret_cast<u16 *>(sh_delta + block_size);
    // sh_times: n_classes * sizeof(u16) bytes
    u16 *sh_rooms = sh_times + n_classes;
    // sh_rooms: n_classes * sizeof(u16) bytes
    u16 *sh_class = sh_rooms + n_classes;
    // sh_class: block_size * sizeof(u16)
    u16 *sh_time = sh_class + block_size;
    // sh_time: block_size * sizeof(u16)
    u16 *sh_room = sh_time + block_size;
    // sh_room: block_size * sizeof(u16)

    for (usize i = tid; i < n_classes; i += block_size) {
        sh_times[i] = pop_times[sol_offset + i];
        sh_rooms[i] = pop_rooms[sol_offset + i];
    }
    __syncthreads();

    curandState rng;
    curand_init(seed, sol * block_size + tid, 0, &rng);
    for (u32 iter = 0; iter < n_iterations; iter++) {
        u16 my_cls = curand(&rng) % n_classes;
        u16 old_t = sh_times[my_cls];
        u16 old_r = sh_rooms[my_cls];

        int2 best_delta = make_int2(0, 0);
        u16 best_time = old_t;
        u16 best_room = old_r;

        u16 t_start = class_times_start[my_cls];
        u16 t_end = class_times_end[my_cls];
        u16 n_time_opts = t_end - t_start;

        u16 r_start = class_rooms_start[my_cls];
        u16 r_end = class_rooms_end[my_cls];
        u16 n_room_opts = r_end - r_start;
        bool needs_room = r_start != r_end;

        for (u32 trial = 0; trial < n_trials; trial++) {
            u16 new_t = t_start + (n_time_opts > 0 ? curand(&rng) % n_time_opts : 0);
            u16 new_r;
            if (!needs_room) {
                new_r = NO_ROOM;
            } else {
                new_r = r_start + curand(&rng) % n_room_opts;
            }
            if (new_t == old_t && new_r == old_r) {
                continue;
            }
            int2 delta = compute_move_delta(my_cls, old_t, old_r, new_t, new_r, sh_times, sh_rooms, n_classes,
                                            time_opt_times, time_opt_penalty, room_opt_room_idx, room_opt_penalty,
                                            room_unavail, room_unavail_offsets, n_rooms, n_unavail, opt_time, opt_room);
            if (cmp_delta(delta, best_delta)) {
                best_delta = delta;
                best_time = new_t;
                best_room = new_r;
            }
        }

        sh_delta[tid] = best_delta;
        sh_class[tid] = my_cls;
        sh_time[tid] = best_time;
        sh_room[tid] = best_room;
        __syncthreads();

        // find the thread with the biggest improvement
        for (u32 s = block_size / 2; s > 0; s /= 2) {
            if (tid < s && cmp_delta(sh_delta[tid + s], sh_delta[tid])) {
                sh_delta[tid] = sh_delta[tid + s];
                sh_class[tid] = sh_class[tid + s];
                sh_time[tid] = sh_time[tid + s];
                sh_room[tid] = sh_room[tid + s];
            }
            __syncthreads();
        }

        // apply the improvement (if any was found)
        if (tid == 0 && cmp_delta(sh_delta[0], NEUTRAL)) {
            u16 c = sh_class[0];
            sh_times[c] = sh_time[0];
            sh_rooms[c] = sh_room[0];
            pop_times[sol_offset + c] = sh_time[0];
            pop_rooms[sol_offset + c] = sh_room[0];
        }
        __syncthreads();
    }
}

void LocalSearch::search(Population &population, const TimetableData &data) {
    if (n_iters == 0) {
        return;
    }

    usize n_classes = population.n_classes;
    u16 *pop_times = thrust::raw_pointer_cast(population.times.data());
    u16 *pop_rooms = thrust::raw_pointer_cast(population.rooms.data());

    const u16 *class_times_start = thrust::raw_pointer_cast(data.classes.times_start.data());
    const u16 *class_times_end = thrust::raw_pointer_cast(data.classes.times_end.data());
    const u16 *class_rooms_start = thrust::raw_pointer_cast(data.classes.rooms_start.data());
    const u16 *class_rooms_end = thrust::raw_pointer_cast(data.classes.rooms_end.data());
    const parser::TimeSlots *time_opt_times = thrust::raw_pointer_cast(data.time_options.times.data());
    const u32 *time_opt_penalty = thrust::raw_pointer_cast(data.time_options.penalty.data());
    const u16 *room_opt_room_idx = thrust::raw_pointer_cast(data.room_options.room_idx.data());
    const u32 *room_opt_penalty = thrust::raw_pointer_cast(data.room_options.penalty.data());
    const parser::TimeSlots *room_unavail = thrust::raw_pointer_cast(data.room_data.unavail.data());
    const usize *room_unavail_offsets = thrust::raw_pointer_cast(data.room_data.unavail_offsets.data());
    usize n_rooms = data.room_data.n_rooms;
    usize n_unavail = data.room_data.unavail.size();
    const auto &opt = data.optimization;

    u32 seed = population.seed ^ static_cast<u32>(rand());
    constexpr u32 block_dim = 1024;
    u32 grid_dim = static_cast<u32>(population.population_size);
    usize sh_mem_size = (2 * n_classes + 3 * block_dim) * sizeof(u16) + block_dim * sizeof(int2);
    k_local_search<<<grid_dim, block_dim, sh_mem_size>>>(
        pop_times, pop_rooms, n_classes, class_times_start, class_times_end, class_rooms_start, class_rooms_end,
        time_opt_times, time_opt_penalty, room_opt_room_idx, room_opt_penalty, room_unavail, room_unavail_offsets,
        n_rooms, n_unavail, opt.time, opt.room, n_iters, n_trials, seed);

    cudaErrCheck(cudaDeviceSynchronize());
}

} // namespace kernels
