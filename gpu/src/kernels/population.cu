#include <curand_kernel.h>
#include <thrust/sequence.h>
#include <thrust/sort.h>

#include "kernels/population.cuh"
#include "kernels/utils.cuh"

namespace kernels {

__global__ void k_init_population(u16 *times, u16 *rooms, usize n_classes, const u16 *sol_indices,
                                  const u16 *times_start, const u16 *times_end, const u16 *rooms_start,
                                  const u16 *rooms_end, const parser::TimeSlots *time_opt_times,
                                  const u16 *room_opt_room_idx, const parser::TimeSlots *room_unavail,
                                  const usize *room_unavail_offsets, usize n_rooms, usize n_unavail, u32 seed) {
    // sol_indices are non-null when this kernel is used to re-init the worst solutions
    usize sol = sol_indices ? static_cast<usize>(sol_indices[blockIdx.x]) : blockIdx.x;
    usize tid = threadIdx.x;
    usize block_size = blockDim.x;
    usize sol_offset = sol * n_classes;

    extern __shared__ u8 sh_mem[];
    auto sh_violations = reinterpret_cast<u32 *>(sh_mem);
    // sh_violations: block_size * sizeof(u32)
    auto sh_times = reinterpret_cast<u16 *>(sh_violations + block_size);
    // sh_times: n_classes * sizeof(u16)
    u16 *sh_rooms = sh_times + n_classes;
    // sh_rooms: n_classes * sizeof(u16)
    u16 *sh_cand_t = sh_rooms + n_classes;
    // sh_cand_t: block_size * sizeof(u16)
    u16 *sh_cand_r = sh_cand_t + block_size;
    // sh_cand_r: block_size * sizeof(u16)

    curandState rng;
    curand_init(seed, sol * block_size + tid, 0, &rng);

    for (usize cls = 0; cls < n_classes; cls++) {
        u16 t_start = times_start[cls];
        u16 t_end = times_end[cls];
        u16 n_times = t_end - t_start;
        u16 r_start = rooms_start[cls];
        u16 r_end = rooms_end[cls];
        bool needs_room = r_start != r_end;

        u16 cand_t = t_start + (n_times > 0 ? curand(&rng) % n_times : 0);
        u16 cand_r;
        if (!needs_room) {
            cand_r = NO_ROOM;
        } else {
            cand_r = r_start + curand(&rng) % (r_end - r_start);
        }

        // count violations against already committed classes
        u32 violations = 0;
        if (cand_r != NO_ROOM) {
            u16 real_room = room_opt_room_idx[cand_r];
            const parser::TimeSlots &cand_time = time_opt_times[cand_t];
            for (usize c2 = 0; c2 < cls; c2++) {
                u16 c2_r = sh_rooms[c2];
                if (c2_r != NO_ROOM && room_opt_room_idx[c2_r] == real_room) {
                    if (utils::timeslots_overlap(cand_time, time_opt_times[sh_times[c2]])) {
                        violations++;
                    }
                }
            }
            usize unavail_start = room_unavail_offsets[real_room];
            usize unavail_end = real_room < n_rooms - 1 ? room_unavail_offsets[real_room + 1] : n_unavail;
            for (usize u = unavail_start; u < unavail_end; u++) {
                if (utils::timeslots_overlap(room_unavail[u], cand_time)) {
                    violations++;
                    break;
                }
            }
        }
        sh_violations[tid] = violations;
        sh_cand_t[tid] = cand_t;
        sh_cand_r[tid] = cand_r;
        __syncthreads();

        // reduction to find the candidate with minimum violations
        for (u32 s = block_size / 2; s > 0; s /= 2) {
            if (tid < s && sh_violations[tid + s] < sh_violations[tid]) {
                sh_violations[tid] = sh_violations[tid + s];
                sh_cand_t[tid] = sh_cand_t[tid + s];
                sh_cand_r[tid] = sh_cand_r[tid + s];
            }
            __syncthreads();
        }
        if (tid == 0) {
            sh_times[cls] = sh_cand_t[0];
            sh_rooms[cls] = sh_cand_r[0];
        }
        __syncthreads();
    }

    for (usize i = tid; i < n_classes; i += block_size) {
        times[sol_offset + i] = sh_times[i];
        rooms[sol_offset + i] = sh_rooms[i];
    }
}

Population::Population(usize n_classes, usize population_size, f32 elites_frac, f32 worst_frac, u32 seed)
    : times(n_classes * population_size), rooms(n_classes * population_size), penalty(population_size),
      order(population_size), seed(seed), n_classes(n_classes), population_size(population_size),
      elites_frac(elites_frac), worst_frac(worst_frac) {}

void Population::init(const TimetableData &d_data) {
    const u16 *d_times_start = thrust::raw_pointer_cast(d_data.classes.times_start.data());
    const u16 *d_times_end = thrust::raw_pointer_cast(d_data.classes.times_end.data());
    const u16 *d_rooms_start = thrust::raw_pointer_cast(d_data.classes.rooms_start.data());
    const u16 *d_rooms_end = thrust::raw_pointer_cast(d_data.classes.rooms_end.data());
    u16 *d_times = thrust::raw_pointer_cast(this->times.data());
    u16 *d_rooms = thrust::raw_pointer_cast(this->rooms.data());

    const parser::TimeSlots *time_opt_times = thrust::raw_pointer_cast(d_data.time_options.times.data());
    const u16 *room_opt_room_idx = thrust::raw_pointer_cast(d_data.room_options.room_idx.data());
    const parser::TimeSlots *room_unavail = thrust::raw_pointer_cast(d_data.room_data.unavail.data());
    const usize *room_unavail_offsets = thrust::raw_pointer_cast(d_data.room_data.unavail_offsets.data());
    usize n_rooms = d_data.room_data.n_rooms;
    usize n_unavail = d_data.room_data.unavail.size();

    u32 seed = this->seed ^ static_cast<u32>(rand());
    constexpr u32 block_dim = 1024;
    u32 grid_dim = static_cast<u32>(population_size);
    usize sh_mem_size = (2 * n_classes + 2 * block_dim) * sizeof(u16) + block_dim * sizeof(u32);
    k_init_population<<<grid_dim, block_dim, sh_mem_size>>>(
        d_times, d_rooms, n_classes, nullptr, d_times_start, d_times_end, d_rooms_start, d_rooms_end, time_opt_times,
        room_opt_room_idx, room_unavail, room_unavail_offsets, n_rooms, n_unavail, seed);
    thrust::sequence(order.begin(), order.end());

    cudaErrCheck(cudaDeviceSynchronize());
}

void Population::replace_worst(const TimetableData &d_data) {
    usize n_worst = std::ceil(population_size * worst_frac);
    // assuming `Population::sort` was called earlier this generation, so that the
    // worst solutions are at the end
    const u16 *d_worst = thrust::raw_pointer_cast(order.data() + population_size - n_worst);

    u16 *d_times = thrust::raw_pointer_cast(times.data());
    u16 *d_rooms = thrust::raw_pointer_cast(rooms.data());
    const u16 *d_times_start = thrust::raw_pointer_cast(d_data.classes.times_start.data());
    const u16 *d_times_end = thrust::raw_pointer_cast(d_data.classes.times_end.data());
    const u16 *d_rooms_start = thrust::raw_pointer_cast(d_data.classes.rooms_start.data());
    const u16 *d_rooms_end = thrust::raw_pointer_cast(d_data.classes.rooms_end.data());

    const parser::TimeSlots *time_opt_times = thrust::raw_pointer_cast(d_data.time_options.times.data());
    const u16 *room_opt_room_idx = thrust::raw_pointer_cast(d_data.room_options.room_idx.data());
    const parser::TimeSlots *room_unavail = thrust::raw_pointer_cast(d_data.room_data.unavail.data());
    const usize *room_unavail_offsets = thrust::raw_pointer_cast(d_data.room_data.unavail_offsets.data());
    usize n_rooms = d_data.room_data.n_rooms;
    usize n_unavail = d_data.room_data.unavail.size();

    u32 seed = this->seed ^ static_cast<u32>(rand());
    constexpr u32 block_dim = 1024;
    u32 grid_dim = static_cast<u32>(n_worst);
    usize sh_mem_size = (2 * n_classes + 2 * block_dim) * sizeof(u16) + block_dim * sizeof(u32);

    k_init_population<<<grid_dim, block_dim, sh_mem_size>>>(
        d_times, d_rooms, n_classes, d_worst, d_times_start, d_times_end, d_rooms_start, d_rooms_end, time_opt_times,
        room_opt_room_idx, room_unavail, room_unavail_offsets, n_rooms, n_unavail, seed);
    cudaErrCheck(cudaDeviceSynchronize());
}

void Population::sort() {
    thrust::sequence(order.begin(), order.end());
    thrust::sort_by_key(penalty.begin(), penalty.end(), order.begin());
}

FoundSolution Population::get_best_solution(const StudentAssignment &assignment) const {
    // assuming `Population::sort` was called earlier this generation
    usize idx = this->order[0];
    Penalty penalty = this->penalty[0];

    std::vector<u16> times_idxs(n_classes), rooms_idxs(n_classes);
    thrust::copy(this->times.begin() + idx * n_classes, this->times.begin() + (idx + 1) * n_classes,
                 times_idxs.begin());
    thrust::copy(this->rooms.begin() + idx * n_classes, this->rooms.begin() + (idx + 1) * n_classes,
                 rooms_idxs.begin());

    std::vector<std::vector<u16>> student_assignment(n_classes);
    std::vector<u32> class_counts(n_classes);
    thrust::copy(assignment.class_counts.begin() + idx * n_classes,
                 assignment.class_counts.begin() + (idx + 1) * n_classes, class_counts.begin());
    for (usize i = 0; i < n_classes; i++) {
        student_assignment[i] = std::vector<u16>(class_counts[i]);
        thrust::copy(assignment.students_idxs.begin() + idx * n_classes * MAX_CLASS_LIMIT + i * MAX_CLASS_LIMIT,
                     assignment.students_idxs.begin() + idx * n_classes * MAX_CLASS_LIMIT + i * MAX_CLASS_LIMIT +
                         class_counts[i],
                     student_assignment[i].begin());
    }

    return {student_assignment, times_idxs, rooms_idxs, penalty};
}

} // namespace kernels
