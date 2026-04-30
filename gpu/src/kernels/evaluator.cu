#include "kernels/evaluator.cuh"

namespace kernels::evaluator {

__device__ bool timeslots_overlap(const parser::TimeSlots &a, const parser::TimeSlots &b) {
    if ((a.weeks.bits & b.weeks.bits) == 0) {
        return false;
    }
    if ((a.days.bits & b.days.bits) == 0) {
        return false;
    }
    return a.start < b.start + b.length && b.start < a.start + a.length;
}

__device__ bool insufficient_travel_time(const parser::TimeSlots &a, const parser::TimeSlots &b, u32 travel) {
    if ((a.weeks.bits & b.weeks.bits) == 0) {
        return false;
    }
    if ((a.days.bits & b.days.bits) == 0) {
        return false;
    }
    u32 a_end = a.start + a.length;
    u32 b_end = b.start + b.length;
    u32 gap;
    if (a_end <= b.start) {
        gap = b.start - a_end;
    } else if (b_end <= a.start) {
        gap = a.start - b_end;
    } else {
        return false; // timeslots overlap
    }

    return gap < travel;
}

__device__ bool is_student_in_class(const u16 *students_idxs, const u32 *class_counts, usize sol, usize cls,
                                    u16 student_idx, usize n_classes) {
    u32 cnt = class_counts[sol * n_classes + cls];
    usize offset = MAX_CLASS_LIMIT * (sol * n_classes + cls);
    for (u32 i = 0; i < cnt; i++) {
        if (students_idxs[offset + i] == student_idx) {
            return true;
        }
    }

    return false;
}

__device__ void apply_dist_penalty(u32 &hard, u32 &soft, const Penalty &p, u32 factor = 1) {
    hard += p.hard * factor;
    soft += p.soft * factor;
}

__global__ void evaluate_kernel(
    Penalty *penalties,
    const u16 *pop_times,
    const u16 *pop_rooms,
    const u16 *students_idxs, const u32 *class_counts,
    const parser::TimeSlots *time_opt_times, const u32 *time_opt_penalty,
    const u16 *room_opt_room_idx, const u32 *room_opt_penalty,
    const u32 *class_limit, const u16 *class_parent,
    const u32 *room_capacity, const parser::TimeSlots *room_unavail, const usize *room_unavail_offsets,
    const u32 *travel_time, usize n_rooms, usize n_unavail,
    const u16 *student_course_idxs, const usize *student_course_offsets,
    const u16 *courses_configs_start, const u16 *courses_configs_end, const u16 *configs_subparts_start,
    const u16 *configs_subparts_end, const u16 *subparts_classes_start, const u16 *subparts_classes_end,
    u32 opt_time, u32 opt_room, u32 opt_student,
    usize n_classes, usize n_students) {
    usize sol = blockIdx.x;
    usize tid = threadIdx.y * blockDim.x + threadIdx.x;
    usize block_size = blockDim.x * blockDim.y;
    usize sol_offset = sol * n_classes;

    __shared__ u32 sh_hard;
    __shared__ u32 sh_soft;
    if (tid == 0) {
        sh_hard = 0;
        sh_soft = 0;
    }
    __syncthreads();

    {
        u32 local_time_pen = 0;
        u32 local_room_pen = 0;
        u32 local_hard = 0;

        for (usize c = tid; c < n_classes; c += block_size) {
            u16 t_opt_idx = pop_times[sol_offset + c];
            u16 r_opt_idx = pop_rooms[sol_offset + c];
            local_time_pen += time_opt_penalty[t_opt_idx];

            // room option penalty
            if (r_opt_idx != NO_ROOM) {
                local_room_pen += room_opt_penalty[r_opt_idx];
            }

            // class limit
            u32 cnt_students = class_counts[sol_offset + c];
            u32 lim = class_limit[c];
            if (lim != NO_LIMIT && cnt_students > lim) {
                local_hard++;
            }

            // room capacity
            if (r_opt_idx != NO_ROOM) {
                u16 room_idx = room_opt_room_idx[r_opt_idx];
                if (room_capacity[room_idx] < cnt_students) {
                    local_hard++;
                }
                const parser::TimeSlots &cls_time = time_opt_times[t_opt_idx];
                usize ua_start = room_unavail_offsets[room_idx];
                usize ua_end = room_idx < n_rooms - 1
                                   ? room_unavail_offsets[room_idx + 1]
                                   : n_unavail;
                for (usize u = ua_start; u < ua_end; u++) {
                    if (timeslots_overlap(room_unavail[u], cls_time)) {
                        local_hard++;
                        break;
                    }
                }
            }
        }
        atomicAdd(&sh_hard, local_hard);
        atomicAdd(&sh_soft, local_time_pen * opt_time + local_room_pen * opt_room);
    }
    __syncthreads();

    // two classes assigned to the same room with overlapping timeslots
    {
        u32 local_hard = 0;
        for (usize idx_i = threadIdx.x; idx_i < n_classes; idx_i += blockDim.x) {
            for (usize idx_j = idx_i + 1; idx_j < n_classes; idx_j += blockDim.y) {
                u16 ri = pop_rooms[sol_offset + idx_i];
                u16 rj = pop_rooms[sol_offset + idx_j];
                if (ri != NO_ROOM && rj != NO_ROOM && room_opt_room_idx[ri] == room_opt_room_idx[rj]) {
                    const parser::TimeSlots &ti = time_opt_times[pop_times[sol_offset + idx_i]];
                    const parser::TimeSlots &tj = time_opt_times[pop_times[sol_offset + idx_j]];
                    if (timeslots_overlap(ti, tj))
                        local_hard++;
                }
            }
        }
        atomicAdd(&sh_hard, local_hard);
    }
    __syncthreads();

    // students not enrolled in parent classes
    {
        u32 local_hard = 0;
        for (usize c = tid; c < n_classes; c += blockDim.x) {
            u16 par = class_parent[c];
            if (par == NO_PARENT) {
                continue;
            }
            u32 cnt = class_counts[sol_offset + c];
            usize offset = MAX_CLASS_LIMIT * (sol * n_classes + c);
            for (u32 k = 0; k < cnt; k++) {
                u16 student_idx = students_idxs[offset + k];
                if (!is_student_in_class(students_idxs, class_counts, sol, par, student_idx, n_classes)) {
                    local_hard++;
                }
            }
        }
        atomicAdd(&sh_hard, local_hard);
    }
    __syncthreads();

    // students not enrolled in exactly one subpart per config
    {
        u32 local_hard = 0;
        for (usize si = tid; si < n_students; si += blockDim.x) {
            usize course_begin = student_course_offsets[si];
            usize course_end = student_course_offsets[si + 1];
            for (usize i = course_begin; i < course_end; i++) {
                u16 course_idx = student_course_idxs[i];
                u16 config_start = courses_configs_start[course_idx];
                u16 config_end = courses_configs_end[course_idx];
                u32 best_penalty = UINT32_MAX;
                for (u16 cfg = config_start; cfg < config_end; cfg++) {
                    u32 penalty = 0;
                    u16 subpart_start = configs_subparts_start[cfg];
                    u16 subpart_end = configs_subparts_end[cfg];
                    for (u16 subpart = subpart_start; subpart < subpart_end; ++subpart) {
                        u16 class_start = subparts_classes_start[subpart];
                        u16 class_end = subparts_classes_end[subpart];
                        u32 assigned = 0;
                        for (u16 cls = class_start; cls < class_end; cls++) {
                            if (is_student_in_class(students_idxs, class_counts, sol, cls, static_cast<u16>(si),
                                                    n_classes)) {
                                assigned++;
                            }
                        }
                        if (assigned == 0) {
                            penalty++;
                        } else if (assigned > 1) {
                            penalty += assigned - 1;
                        }
                    }
                    if (penalty < best_penalty) {
                        best_penalty = penalty;
                    }
                }
                if (best_penalty != UINT32_MAX) {
                    local_hard += best_penalty;
                }
            }
        }
        atomicAdd(&sh_hard, local_hard);
    }
    __syncthreads();

    // conflits among a single student's assignments
    {
        u32 local_conflicts = 0;
        for (usize si = tid; si < n_students; si += blockDim.x) {
            constexpr u16 MAX_ATTEND = 128; // should be enough
            u16 attending[MAX_ATTEND];
            u16 n_att = 0;
            for (usize c = 0; c < n_classes && n_att < MAX_ATTEND; c++) {
                if (is_student_in_class(students_idxs, class_counts, sol, c, static_cast<u16>(si), n_classes)) {
                    attending[n_att++] = static_cast<u16>(c);
                }
            }
            for (u16 a = 0; a < n_att; a++) {
                u16 ca = attending[a];
                u16 ta = pop_times[sol_offset + ca];
                u16 ra = pop_rooms[sol_offset + ca];
                const parser::TimeSlots &time_a = time_opt_times[ta];
                for (u16 b = a + 1; b < n_att; ++b) {
                    u16 cb = attending[b];
                    u16 tb = pop_times[sol_offset + cb];
                    u16 rb = pop_rooms[sol_offset + cb];
                    const parser::TimeSlots &time_b = time_opt_times[tb];
                    if (timeslots_overlap(time_a, time_b)) {
                        local_conflicts++;
                    } else if (ra != NO_ROOM && rb != NO_ROOM) {
                        u16 room_a = room_opt_room_idx[ra];
                        u16 room_b = room_opt_room_idx[rb];
                        u32 travel = travel_time[room_a * n_rooms + room_b];
                        if (travel != NO_TRAVEL && travel > 0 && insufficient_travel_time(time_a, time_b, travel)) {
                            local_conflicts++;
                        }
                    }
                }
            }
        }
        atomicAdd(&sh_soft, local_conflicts * opt_student);
    }
    __syncthreads();

    // TODO: distribution penalties

    if (tid == 0) {
        penalties[sol] = Penalty(sh_hard, sh_soft);
    }
}

void evaluate(const TimetableData &d_data, Population &population, const StudentAssignment &assignment) {
    const usize n_classes = population.n_classes;

    const u16 *d_pop_times = thrust::raw_pointer_cast(population.times.data());
    const u16 *d_pop_rooms = thrust::raw_pointer_cast(population.rooms.data());
    Penalty *d_penalties =
        thrust::raw_pointer_cast(population.penalty.data());

    const u16 *d_student_idxs = thrust::raw_pointer_cast(assignment.students_idxs.data());
    const u32 *d_class_counts = thrust::raw_pointer_cast(assignment.class_counts.data());

    const parser::TimeSlots *time_opt_times = thrust::raw_pointer_cast(d_data.time_options.times.data());
    const u32 *time_opt_penalty = thrust::raw_pointer_cast(d_data.time_options.penalty.data());

    const u16 *room_opt_room_idx = thrust::raw_pointer_cast(d_data.room_options.room_idx.data());
    const u32 *room_opt_penalty = thrust::raw_pointer_cast(d_data.room_options.penalty.data());

    const u32 *d_limit = thrust::raw_pointer_cast(d_data.classes.limit.data());
    const u16 *d_parent = thrust::raw_pointer_cast(d_data.classes.parent.data());

    const u32 *d_room_capacity = thrust::raw_pointer_cast(d_data.room_data.capacity.data());
    const parser::TimeSlots *d_room_unavail = thrust::raw_pointer_cast(d_data.room_data.unavail.data());
    const usize *d_room_unavail_offsets = thrust::raw_pointer_cast(d_data.room_data.unavail_offsets.data());
    const u32 *d_travel = thrust::raw_pointer_cast(d_data.room_data.travel_time.data());
    const usize n_rooms = d_data.room_data.n_rooms;
    const usize n_unavail = d_data.room_data.unavail.size();

    const u16 *d_sc_idxs = thrust::raw_pointer_cast(d_data.students.course_idxs.data());
    const usize *d_sc_offsets = thrust::raw_pointer_cast(d_data.students.course_idxs_offsets.data());
    const usize n_students = d_data.students.id.size();

    const u16 *d_cc_start = thrust::raw_pointer_cast(d_data.courses.configs_start.data());
    const u16 *d_cc_end = thrust::raw_pointer_cast(d_data.courses.configs_end.data());
    const u16 *d_cs_start = thrust::raw_pointer_cast(d_data.configs.subparts_start.data());
    const u16 *d_cs_end = thrust::raw_pointer_cast(d_data.configs.subparts_end.data());
    const u16 *d_sc_start = thrust::raw_pointer_cast(d_data.subparts.classes_start.data());
    const u16 *d_sc_end = thrust::raw_pointer_cast(d_data.subparts.classes_end.data());
    const auto &opt = d_data.optimization;

    constexpr dim3 block_dim(32, 32);
    dim3 grid_dim(static_cast<u32>(population.population_size));
    evaluate_kernel<<<grid_dim, block_dim>>>(
        d_penalties, d_pop_times, d_pop_rooms, d_student_idxs, d_class_counts, time_opt_times, time_opt_penalty, room_opt_room_idx,
        room_opt_penalty,
        d_limit, d_parent, d_room_capacity, d_room_unavail, d_room_unavail_offsets, d_travel, n_rooms, n_unavail, d_sc_idxs, d_sc_offsets,
        d_cc_start, d_cc_end,
        d_cs_start, d_cs_end, d_sc_start, d_sc_end, opt.time, opt.room,
        opt.student, n_classes, n_students);

    cudaErrCheck(cudaDeviceSynchronize());
}

}