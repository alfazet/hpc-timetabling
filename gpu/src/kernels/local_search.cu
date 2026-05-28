#include <curand_kernel.h>

#include "kernels/local_search.cuh"
#include "kernels/utils.cuh"

constexpr int2 NEUTRAL{0, 0};

namespace kernels {

LocalSearch::LocalSearch(u32 n_iters) : n_iters(n_iters) {}

__device__ inline bool cmp_delta(int2 a, int2 b) {
    if (a.x == b.x) {
        return a.y < b.y;
    }
    return a.x < b.x;
}

// checks whether a pairwise distribution constraint is violated for a given pair of classes
__device__ static bool dist_pair_violated(DistributionKind kind, const parser::TimeSlots &ti,
                                          const parser::TimeSlots &tj, u16 ri, u16 rj, const u16 *room_opt_room_idx,
                                          const u32 *travel_time, usize n_rooms) {
    u8 di = ti.days.bits, dj = tj.days.bits;
    u16 wi = ti.weeks.bits, wj = tj.weeks.bits;
    if (cuda::std::holds_alternative<parser::SameStart>(kind)) {
        return ti.start != tj.start;
    }
    if (cuda::std::holds_alternative<parser::SameTime>(kind)) {
        bool contains = (tj.start <= ti.start && ti.start + ti.length <= tj.start + tj.length) ||
                        (ti.start <= tj.start && tj.start + tj.length <= ti.start + ti.length);
        return !contains;
    }
    if (cuda::std::holds_alternative<parser::DifferentTime>(kind)) {
        return !(tj.start + tj.length <= ti.start || ti.start + ti.length <= tj.start);
    }
    if (cuda::std::holds_alternative<parser::SameDays>(kind)) {
        return !((dj | di) == dj || (dj | di) == di);
    }
    if (cuda::std::holds_alternative<parser::DifferentDays>(kind)) {
        return (di & dj) != 0;
    }
    if (cuda::std::holds_alternative<parser::SameWeeks>(kind)) {
        return !((wj | wi) == wj || (wj | wi) == wi);
    }
    if (cuda::std::holds_alternative<parser::DifferentWeeks>(kind)) {
        return (wi & wj) != 0;
    }
    if (cuda::std::holds_alternative<parser::Overlap>(kind) || cuda::std::holds_alternative<parser::NotOverlap>(kind)) {
        bool overlap =
            tj.start < ti.start + ti.length && ti.start < tj.start + tj.length && (di & dj) != 0 && (wi & wj) != 0;
        bool should_overlap = cuda::std::holds_alternative<parser::Overlap>(kind);
        return (should_overlap && !overlap) || (!should_overlap && overlap);
    }
    if (cuda::std::holds_alternative<parser::SameRoom>(kind) ||
        cuda::std::holds_alternative<parser::DifferentRoom>(kind)) {
        bool same_room = (ri == NO_ROOM && rj == NO_ROOM) ||
                         (ri != NO_ROOM && rj != NO_ROOM && room_opt_room_idx[ri] == room_opt_room_idx[rj]);
        bool should_same = cuda::std::holds_alternative<parser::SameRoom>(kind);
        return (should_same && !same_room) || (!should_same && same_room);
    }
    if (cuda::std::holds_alternative<parser::SameAttendees>(kind)) {
        if (ri == NO_ROOM || rj == NO_ROOM) {
            return false;
        }
        if ((di & dj) == 0 || (wi & wj) == 0) {
            return false;
        }
        u16 room_i = room_opt_room_idx[ri];
        u16 room_j = room_opt_room_idx[rj];
        u32 travel_ij = travel_time[room_i * n_rooms + room_j];
        u32 travel_ji = travel_time[room_j * n_rooms + room_i];
        if (travel_ij == NO_TRAVEL) {
            travel_ij = 0;
        }
        if (travel_ji == NO_TRAVEL) {
            travel_ji = 0;
        }
        u32 travel = max(travel_ij, travel_ji);
        bool ok = tj.start + tj.length + travel <= ti.start || ti.start + ti.length + travel <= tj.start;
        return !ok;
    }
    if (cuda::std::holds_alternative<parser::Precedence>(kind)) {
        u16 _wi = __ffs(wi) - 1, _wj = __ffs(wj) - 1;
        u16 _di = __ffs(di) - 1, _dj = __ffs(dj) - 1;
        bool ok = _wi < _wj || (_wi == _wj && (_di < _dj || (_di == _dj && ti.start + ti.length <= tj.start)));
        return !ok;
    }
    if (cuda::std::holds_alternative<parser::WorkDay>(kind)) {
        if ((di & dj) == 0 || (wi & wj) == 0) {
            return false;
        }
        u32 span = max(ti.start + ti.length, tj.start + tj.length) - min(ti.start, tj.start);
        return span > cuda::std::get<parser::WorkDay>(kind).s;
    }
    if (cuda::std::holds_alternative<parser::MinGap>(kind)) {
        if ((di & dj) == 0 || (wi & wj) == 0) {
            return false;
        }
        u16 g = cuda::std::get<parser::MinGap>(kind).g;
        return ti.start + ti.length + g > tj.start && tj.start + tj.length + g > ti.start;
    }
    return false;
}

// compute distribution penalty delta when moving class `cls` from old_t/old_r to new_t/new_r
__device__ static int2 compute_dist_delta(usize cls, u16 old_t, u16 old_r, u16 new_t, u16 new_r, const u16 *sh_times,
                                          const u16 *sh_rooms, const parser::TimeSlots *time_opt_times,
                                          const u16 *room_opt_room_idx, const u32 *travel_time, usize n_rooms,
                                          const DistributionKind *dist_kind, const u16 *dist_class_idxs,
                                          const usize *dist_class_idxs_offsets, const Penalty *dist_penalty,
                                          const u16 *class_dist_idxs, const usize *class_dist_offsets) {
    i32 delta_hard = 0;
    i32 delta_soft = 0;

    const parser::TimeSlots &old_time = time_opt_times[old_t];
    const parser::TimeSlots &new_time = time_opt_times[new_t];
    usize d_begin = class_dist_offsets[cls];
    usize d_end = class_dist_offsets[cls + 1];
    for (usize di = d_begin; di < d_end; di++) {
        u16 d = class_dist_idxs[di];
        DistributionKind kind = dist_kind[d];
        Penalty pen = dist_penalty[d];

        // skip these because they don't make sense to check in this context
        if (cuda::std::holds_alternative<parser::MaxDays>(kind) ||
            cuda::std::holds_alternative<parser::MaxDayLoad>(kind) ||
            cuda::std::holds_alternative<parser::MaxBreaks>(kind) ||
            cuda::std::holds_alternative<parser::MaxBlock>(kind)) {
            continue;
        }

        usize c_begin = dist_class_idxs_offsets[d];
        usize c_end = dist_class_idxs_offsets[d + 1];
        for (usize ci = c_begin; ci < c_end; ci++) {
            u16 c2 = dist_class_idxs[ci];
            if (c2 == cls) {
                continue;
            }

            const parser::TimeSlots &c2_time = time_opt_times[sh_times[c2]];
            u16 c2_r = sh_rooms[c2];
            bool old_violated =
                dist_pair_violated(kind, old_time, c2_time, old_r, c2_r, room_opt_room_idx, travel_time, n_rooms);
            bool new_violated =
                dist_pair_violated(kind, new_time, c2_time, new_r, c2_r, room_opt_room_idx, travel_time, n_rooms);

            if (old_violated && !new_violated) {
                delta_hard -= pen.hard;
                delta_soft -= pen.soft;
            } else if (!old_violated && new_violated) {
                delta_hard += pen.hard;
                delta_soft += pen.soft;
            }
        }
    }

    return make_int2(delta_hard, delta_soft);
}

// checks how the move affects time/room conflicts and distribution penalties
// returns (hard_delta, soft_delta), negative values are improvements
__device__ static int2
compute_move_delta(usize cls, u16 old_t, u16 old_r, u16 new_t, u16 new_r, const u16 *sh_times, const u16 *sh_rooms,
                   usize n_classes, const parser::TimeSlots *time_opt_times, const u32 *time_opt_penalty,
                   const u16 *room_class_idxs, const usize *room_class_offsets, const u16 *room_opt_room_idx,
                   const u32 *room_opt_penalty, const parser::TimeSlots *room_unavail,
                   const usize *room_unavail_offsets, usize n_rooms, usize n_unavail, u32 opt_time, u32 opt_room,
                   const u32 *travel_time, const DistributionKind *dist_kind, const u16 *dist_class_idxs,
                   const usize *dist_class_idxs_offsets, const Penalty *dist_penalty, usize n_distributions,
                   const u16 *class_dist_idxs, const usize *class_dist_offsets, usize n_dist_class_idxs) {
    i32 delta_hard = 0;
    i32 delta_soft = 0;

    delta_soft = (time_opt_penalty[new_t] - time_opt_penalty[old_t]) * opt_time;
    i32 old_room_pen = old_r != NO_ROOM ? room_opt_penalty[old_r] : 0;
    i32 new_room_pen = new_r != NO_ROOM ? room_opt_penalty[new_r] : 0;
    delta_soft += (new_room_pen - old_room_pen) * opt_room;

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

    if (old_real_room != NO_ROOM) {
        u32 start = room_class_offsets[old_real_room];
        u32 end = room_class_offsets[old_real_room + 1];

        for (u32 i = start; i < end; i++) {
            u16 c2 = room_class_idxs[i];

            if (c2 == cls)
                continue;

            u16 c2_r = sh_rooms[c2];
            if (c2_r == NO_ROOM)
                continue;

            u16 c2_real_room = room_opt_room_idx[c2_r];
            const parser::TimeSlots &c2_time = time_opt_times[sh_times[c2]];

            if (c2_real_room == old_real_room && utils::timeslots_overlap(old_time, c2_time)) {
                delta_hard--;
            }
        }
    }

    if (new_real_room != NO_ROOM) {
        u32 start = room_class_offsets[new_real_room];
        u32 end = room_class_offsets[new_real_room + 1];

        for (u32 i = start; i < end; i++) {
            u16 c2 = room_class_idxs[i];

            if (c2 == cls)
                continue;

            u16 c2_r = sh_rooms[c2];
            if (c2_r == NO_ROOM)
                continue;

            u16 c2_real_room = room_opt_room_idx[c2_r];
            const parser::TimeSlots &c2_time = time_opt_times[sh_times[c2]];

            if (c2_real_room == new_real_room && utils::timeslots_overlap(new_time, c2_time)) {
                delta_hard++;
            }
        }
    }

    int2 dist_d = compute_dist_delta(cls, old_t, old_r, new_t, new_r, sh_times, sh_rooms, time_opt_times,
                                     room_opt_room_idx, travel_time, n_rooms, dist_kind, dist_class_idxs,
                                     dist_class_idxs_offsets, dist_penalty, class_dist_idxs, class_dist_offsets);
    delta_hard += dist_d.x;
    delta_soft += dist_d.y;

    return make_int2(delta_hard, delta_soft);
}

__global__ void k_local_search(u16 *pop_times, u16 *pop_rooms, usize n_classes, const u16 *class_times_start,
                               const u16 *class_times_end, const u16 *class_rooms_start, const u16 *class_rooms_end,
                               const u16 *room_class_idxs, const usize *room_class_offsets,
                               const parser::TimeSlots *time_opt_times, const u32 *time_opt_penalty,
                               const u16 *room_opt_room_idx, const u32 *room_opt_penalty,
                               const parser::TimeSlots *room_unavail, const usize *room_unavail_offsets, usize n_rooms,
                               usize n_unavail, u32 opt_time, u32 opt_room, u32 n_iterations, u32 seed,
                               const u32 *travel_time, const DistributionKind *dist_kind, const u16 *dist_class_idxs,
                               const usize *dist_class_idxs_offsets, const Penalty *dist_penalty, usize n_distributions,
                               const u16 *class_dist_idxs, const usize *class_dist_offsets, usize n_dist_class_idxs) {
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

    int2 overall_best_delta = NEUTRAL;
    for (u32 iter = 0; iter < n_iterations; iter++) {
        // TODO: instead of randomly choosing a class to try to fix,
        // choose the one that creates the most violations (similar to mutations)

        // search for a better timeslot
        {
            u16 my_cls = curand(&rng) % n_classes;
            u16 old_t = sh_times[my_cls];
            u16 old_r = sh_rooms[my_cls];
            int2 best_delta = make_int2(0, 0);
            u16 best_time = old_t;
            u16 t_start = class_times_start[my_cls];
            u16 t_end = class_times_end[my_cls];
            u16 n_time_opts = t_end - t_start;

            for (u16 t_idx = 0; t_idx < n_time_opts; t_idx++) {
                u16 new_t = t_start + t_idx;
                if (new_t == old_t) {
                    continue;
                }
                int2 delta = compute_move_delta(my_cls, old_t, old_r, new_t, old_r, sh_times, sh_rooms, n_classes,
                                                time_opt_times, time_opt_penalty, room_class_idxs, room_class_offsets,
                                                room_opt_room_idx, room_opt_penalty, room_unavail, room_unavail_offsets,
                                                n_rooms, n_unavail, opt_time, opt_room, travel_time, dist_kind,
                                                dist_class_idxs, dist_class_idxs_offsets, dist_penalty, n_distributions,
                                                class_dist_idxs, class_dist_offsets, n_dist_class_idxs);
                if (cmp_delta(delta, best_delta)) {
                    best_delta = delta;
                    best_time = new_t;
                }
            }

            sh_delta[tid] = best_delta;
            sh_class[tid] = my_cls;
            sh_time[tid] = best_time;
            sh_room[tid] = old_r;
            __syncthreads();

            for (u32 s = block_size / 2; s > 0; s /= 2) {
                if (tid < s && cmp_delta(sh_delta[tid + s], sh_delta[tid])) {
                    sh_delta[tid] = sh_delta[tid + s];
                    sh_class[tid] = sh_class[tid + s];
                    sh_time[tid] = sh_time[tid + s];
                    sh_room[tid] = sh_room[tid + s];
                }
                __syncthreads();
            }

            if (tid == 0 && cmp_delta(sh_delta[0], NEUTRAL)) {
                u16 c = sh_class[0];
                sh_times[c] = sh_time[0];
                pop_times[sol_offset + c] = sh_time[0];
            }
            __syncthreads();
        }

        // // search for a better room
        {
            u16 my_cls = curand(&rng) % n_classes;
            u16 old_t = sh_times[my_cls];
            u16 old_r = sh_rooms[my_cls];
            u16 r_start = class_rooms_start[my_cls];
            u16 r_end = class_rooms_end[my_cls];
            u16 n_room_opts = r_end - r_start;
            bool needs_room = r_start != r_end;
            int2 best_delta = make_int2(0, 0);
            u16 best_room = old_r;

            if (needs_room) {
                for (u16 r_idx = 0; r_idx < n_room_opts; r_idx++) {
                    u16 new_r = r_start + r_idx;
                    if (new_r == old_r) {
                        continue;
                    }

                    int2 delta = compute_move_delta(
                        my_cls, old_t, old_r, old_t, new_r, sh_times, sh_rooms, n_classes, time_opt_times,
                        time_opt_penalty, room_class_idxs, room_class_offsets, room_opt_room_idx, room_opt_penalty,
                        room_unavail, room_unavail_offsets, n_rooms, n_unavail, opt_time, opt_room, travel_time,
                        dist_kind, dist_class_idxs, dist_class_idxs_offsets, dist_penalty, n_distributions,
                        class_dist_idxs, class_dist_offsets, n_dist_class_idxs);
                    if (cmp_delta(delta, best_delta)) {
                        best_delta = delta;
                        best_room = new_r;
                    }
                }
            }

            sh_delta[tid] = best_delta;
            sh_class[tid] = my_cls;
            sh_time[tid] = old_t;
            sh_room[tid] = best_room;
            __syncthreads();

            for (u32 s = block_size / 2; s > 0; s /= 2) {
                if (tid < s && cmp_delta(sh_delta[tid + s], sh_delta[tid])) {
                    sh_delta[tid] = sh_delta[tid + s];
                    sh_class[tid] = sh_class[tid + s];
                    sh_time[tid] = sh_time[tid + s];
                    sh_room[tid] = sh_room[tid + s];
                }
                __syncthreads();
            }

            if (tid == 0 && cmp_delta(sh_delta[0], overall_best_delta)) {
                overall_best_delta = sh_delta[0];
                u16 c = sh_class[0];
                sh_rooms[c] = sh_room[0];
                pop_rooms[sol_offset + c] = sh_room[0];
            }
            __syncthreads();
        }
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
    const u16 *room_class_idxs = thrust::raw_pointer_cast(data.classes.room_class_idxs.data());
    const usize *room_class_offsets = thrust::raw_pointer_cast(data.classes.room_class_offsets.data());
    const parser::TimeSlots *time_opt_times = thrust::raw_pointer_cast(data.time_options.times.data());
    const u32 *time_opt_penalty = thrust::raw_pointer_cast(data.time_options.penalty.data());
    const u16 *room_opt_room_idx = thrust::raw_pointer_cast(data.room_options.room_idx.data());
    const u32 *room_opt_penalty = thrust::raw_pointer_cast(data.room_options.penalty.data());
    const parser::TimeSlots *room_unavail = thrust::raw_pointer_cast(data.room_data.unavail.data());
    const usize *room_unavail_offsets = thrust::raw_pointer_cast(data.room_data.unavail_offsets.data());
    usize n_rooms = data.room_data.n_rooms;
    usize n_unavail = data.room_data.unavail.size();
    const auto &opt = data.optimization;

    const u32 *travel_time = thrust::raw_pointer_cast(data.room_data.travel_time.data());
    const auto &dist = data.distributions;
    const DistributionKind *dist_kind = thrust::raw_pointer_cast(dist.kind.data());
    const u16 *dist_class_idxs = thrust::raw_pointer_cast(dist.class_idxs.data());
    const usize *dist_class_idxs_offsets = thrust::raw_pointer_cast(dist.class_idxs_offsets.data());
    const Penalty *dist_penalty = thrust::raw_pointer_cast(dist.penalty.data());
    usize n_distributions = dist.kind.size();
    const u16 *class_dist_idxs = thrust::raw_pointer_cast(dist.class_dist_idxs.data());
    const usize *class_dist_offsets = thrust::raw_pointer_cast(dist.class_dist_offsets.data());
    usize n_dist_class_idxs = dist.class_idxs.size();

    u32 seed = population.seed ^ static_cast<u32>(rand());
    constexpr u32 block_dim = SMALL_BLOCK_SIZE;
    u32 grid_dim = static_cast<u32>(population.population_size);
    usize sh_mem_size = (2 * n_classes + 3 * block_dim) * sizeof(u16) + block_dim * sizeof(int2);
    k_local_search<<<grid_dim, block_dim, sh_mem_size>>>(
        pop_times, pop_rooms, n_classes, class_times_start, class_times_end, class_rooms_start, class_rooms_end,
        room_class_idxs, room_class_offsets, time_opt_times, time_opt_penalty, room_opt_room_idx, room_opt_penalty,
        room_unavail, room_unavail_offsets, n_rooms, n_unavail, opt.time, opt.room, n_iters, seed, travel_time,
        dist_kind, dist_class_idxs, dist_class_idxs_offsets, dist_penalty, n_distributions, class_dist_idxs,
        class_dist_offsets, n_dist_class_idxs);

    cudaErrCheck(cudaDeviceSynchronize());
}

} // namespace kernels
