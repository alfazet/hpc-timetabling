#include "kernels/assigner.cuh"

namespace kernels {

__device__ bool timeslots_overlap(const parser::TimeSlots &a, const parser::TimeSlots &b) {
    if ((a.weeks.bits & b.weeks.bits) == 0) {
        return false;
    }
    if ((a.days.bits & b.days.bits) == 0) {
        return false;
    }
    return a.start < b.start + b.length && b.start < a.start + a.length;
}

__global__ void k_assign_students(u16 *students_idxs, u32 *class_counts, const u16 *pop_times,
                                  const u16 *courses_configs_start, const u16 *courses_configs_end,
                                  const u16 *configs_subparts_start, const u16 *configs_subparts_end,
                                  const u16 *subparts_classes_start, const u16 *subparts_classes_end,
                                  const u32 *class_limit, const u16 *class_parent, const u16 *class_subpart_idx,
                                  const parser::TimeSlots *time_opt_times, const u16 *student_course_idxs,
                                  const usize *student_course_offsets, u16 n_classes, u16 n_students) {
    usize tid = threadIdx.x;

    extern __shared__ u8 sh_mem[];
    auto sh_sol_offset = reinterpret_cast<usize*>(sh_mem);
    // sh_sol_offset: sizeof(usize) bytes
    auto *sh_class_count = reinterpret_cast<u16*>(sh_sol_offset + 1);
    // sh_class_count: n_classes * sizeof(u16) bytes
    u16 *sh_trial = sh_class_count + n_classes;
    // sh_trial: MAX_SUBPARTS * sizeof(u16) bytes
    u16 *sh_n_subparts = sh_trial + MAX_SUBPARTS;
    // sh_n_subparts: sizeof(u16) bytes
    auto sh_already_attending = reinterpret_cast<bool*>(sh_n_subparts + 1);
    // sh_already_attending = n_classes * sizeof(bool) bytes
    bool* sh_conflict = sh_already_attending + n_classes;
    // sh_conflict: sizeof(bool) bytes
    bool* sh_course_assigned = sh_conflict + 1;
    // sh_course_assigned = sizeof(bool) bytes
    bool* sh_needs_conflict_check = sh_course_assigned + 1;
    // sh_needs_conflict_check = sizeof(bool) bytes
    bool* sh_done = sh_needs_conflict_check + 1;
    // sh_done = sizeof(bool) bytes

    for (usize i = tid; i < n_classes; i += blockDim.x) {
        sh_class_count[i] = 0;
    }
    __syncthreads();

    usize sol = blockIdx.x;
    usize sol_offset = sol * n_classes;
    if (tid == 0) {
        *sh_sol_offset = sol_offset;
    }
    __syncthreads();

    // class index chosen per subpart in a given config
    u16 local_assignment[MAX_SUBPARTS];
    for (usize student_idx = 0; student_idx < n_students; student_idx++) {
        for (usize i = tid; i < n_classes; i += blockDim.x) {
            sh_already_attending[i] = false;
            usize cnt = sh_class_count[i];
            usize offset = MAX_CLASS_LIMIT * (sol * n_classes + i);
            for (usize j = 0; j < cnt; j++) {
                if (students_idxs[offset + j] == student_idx) {
                    sh_already_attending[i] = true;
                    break;
                }
            }
        }
        __syncthreads();

        usize courses_start = student_course_offsets[student_idx];
        usize courses_end = student_course_offsets[student_idx + 1];
        for (usize i = courses_start; i < courses_end; i++) {
            u16 course_idx = student_course_idxs[i];
            u16 configs_start = courses_configs_start[course_idx];
            u16 configs_end = courses_configs_end[course_idx];

            if (tid == 0) {
                *sh_course_assigned = false;
            }
            __syncthreads();

            for (u16 config_idx = configs_start; config_idx < configs_end; config_idx++) {
                if (*sh_course_assigned) {
                    break;
                }
                u16 n_subparts;
                if (tid == 0) {
                    u16 subparts_start = configs_subparts_start[config_idx];
                    u16 subparts_end = configs_subparts_end[config_idx];
                    n_subparts = subparts_end - subparts_start;
                    *sh_n_subparts = n_subparts;
                    for (u16 j = 0; j < n_subparts && j < MAX_SUBPARTS; j++) {
                        local_assignment[j] = NO_CLASS_ASSIGNED;
                    }
                }
                __syncthreads();
                n_subparts = *sh_n_subparts;
                bool config_ok = true;

                for (u16 sp = 0; sp < n_subparts; sp++) {
                    u16 classes_start_val, classes_end_val;
                    bool subpart_assigned = false;
                    if (tid == 0) {
                        u16 subpart_idx = configs_subparts_start[config_idx] + sp;
                        classes_start_val = subparts_classes_start[subpart_idx];
                        classes_end_val = subparts_classes_end[subpart_idx];
                    }

                    u16 n_candidates = 0;
                    if (tid == 0) {
                        n_candidates = classes_end_val - classes_start_val;
                    }
                    for (u16 ci = 0;; ci++) {
                        if (tid == 0) {
                            *sh_needs_conflict_check = false;
                            *sh_done = ci >= n_candidates || subpart_assigned;
                            if (!*sh_done) {
                                u16 class_idx = classes_start_val + ci;
                                u16 trial[MAX_SUBPARTS];
                                for (u16 j = 0; j < n_subparts; j++) {
                                    trial[j] = local_assignment[j];
                                }
                                u16 cur = class_idx;
                                // resolve parents
                                while (true) {
                                    u16 subpart_offset_val =
                                        class_subpart_idx[cur] - configs_subparts_start[config_idx];
                                    if (subpart_offset_val < MAX_SUBPARTS) {
                                        trial[subpart_offset_val] = cur;
                                    }
                                    u16 par = class_parent[cur];
                                    if (par == NO_PARENT) {
                                        break;
                                    }
                                    cur = par;
                                }

                                // check limits
                                bool ok = true;
                                for (u16 j = 0; j < n_subparts && ok; j++) {
                                    if (trial[j] == NO_CLASS_ASSIGNED || sh_already_attending[trial[j]]) {
                                        continue;
                                    }
                                    u32 limit = class_limit[trial[j]];
                                    if (limit != NO_LIMIT && sh_class_count[trial[j]] + 1 > limit) {
                                        ok = false;
                                    }
                                }

                                if (ok) {
                                    // check time conflicts with already assigned classes
                                    for (u16 j = 0; j < n_subparts && ok; j++) {
                                        if (trial[j] == NO_CLASS_ASSIGNED || sh_already_attending[trial[j]]) {
                                            continue;
                                        }
                                        const parser::TimeSlots &trial_time = time_opt_times[pop_times[sol_offset + trial[j]]];
                                        for (u16 k = 0; k < j && ok; k++) {
                                            if (trial[k] == NO_CLASS_ASSIGNED || sh_already_attending[trial[k]]) {
                                                continue;
                                            }
                                            u16 t = pop_times[sol_offset + trial[k]];
                                            const parser::TimeSlots &time = time_opt_times[t];
                                            if (timeslots_overlap(trial_time, time)) {
                                                ok = false;
                                            }
                                        }
                                    }
                                }

                                if (ok) {
                                    // broadcast to shared memory for parallel conflict check
                                    for (u16 j = 0; j < n_subparts; j++) {
                                        sh_trial[j] = trial[j];
                                    }
                                    *sh_needs_conflict_check = true;
                                    *sh_conflict = false;
                                }
                            }
                        }
                        __syncthreads();
                        // all threads exit together when we exhausted the candidates or assigned the subpart
                        if (*sh_done) {
                            break;
                        }

                        if (*sh_needs_conflict_check) {
                            for (u16 k = tid; k < n_classes; k += blockDim.x) {
                                if (!sh_already_attending[k]) {
                                    continue;
                                }
                                const parser::TimeSlots &at_time = time_opt_times[pop_times[sol_offset + k]];
                                for (u16 j = 0; j < n_subparts; j++) {
                                    if (sh_trial[j] == NO_CLASS_ASSIGNED || sh_already_attending[sh_trial[j]]) {
                                        continue;
                                    }
                                    const parser::TimeSlots &trial_time = time_opt_times[pop_times[sol_offset + sh_trial[j]]];
                                    if (timeslots_overlap(trial_time, at_time)) {
                                        *sh_conflict = true;
                                    }
                                }
                            }
                            __syncthreads();

                            if (tid == 0) {
                                if (!*sh_conflict) {
                                    for (u16 j = 0; j < n_subparts; j++) {
                                        local_assignment[j] = sh_trial[j];
                                    }
                                    subpart_assigned = true;
                                }
                            }
                            __syncthreads();
                        }
                    }

                    if (tid == 0 && !subpart_assigned) {
                        config_ok = false;
                        *sh_done = true;
                    }
                    __syncthreads();
                    if (*sh_done) {
                        if (tid == 0) {
                            // reset for the next iteration
                            *sh_done = false;
                        }
                        __syncthreads();
                        break;
                    }
                }

                // commit the assignment if config_ok
                if (tid == 0 && config_ok) {
                    u16 subparts_start = configs_subparts_start[config_idx];
                    u16 subparts_end = configs_subparts_end[config_idx];
                    u16 n_sp = subparts_end - subparts_start;
                    for (u16 j = 0; j < n_sp; j++) {
                        u16 c = local_assignment[j];
                        if (c == NO_CLASS_ASSIGNED || sh_already_attending[c]) {
                            continue;
                        }
                        u16 cnt = sh_class_count[c];
                        usize offset = MAX_CLASS_LIMIT * (sol * n_classes + c);
                        students_idxs[offset + cnt] = student_idx;
                        sh_class_count[c] = cnt + 1;
                        sh_already_attending[c] = true;
                    }
                    *sh_course_assigned = true;
                }
                __syncthreads();
            }
        }
        __syncthreads();
    }
    for (u16 i = tid; i < n_classes; i += blockDim.x) {
        class_counts[sol_offset + i] = sh_class_count[i];
    }
}

StudentAssignment::StudentAssignment(usize n_classes, usize population_size)
    : students_idxs(n_classes * population_size * MAX_CLASS_LIMIT), class_counts(n_classes * population_size),
      n_classes(n_classes), population_size(population_size) {}

void StudentAssignment::assign(const TimetableData &d_data, const Population &population) {
    thrust::fill(students_idxs.begin(), students_idxs.end(), 0);
    thrust::fill(class_counts.begin(), class_counts.end(), 0);

    u16 *d_students_idxs = thrust::raw_pointer_cast(this->students_idxs.data());
    u32 *d_class_counts = thrust::raw_pointer_cast(this->class_counts.data());

    const u16 *d_pop_times = thrust::raw_pointer_cast(population.times.data());
    const u16 *d_courses_configs_start = thrust::raw_pointer_cast(d_data.courses.configs_start.data());
    const u16 *d_courses_configs_end = thrust::raw_pointer_cast(d_data.courses.configs_end.data());
    const u16 *d_configs_subparts_start = thrust::raw_pointer_cast(d_data.configs.subparts_start.data());
    const u16 *d_configs_subparts_end = thrust::raw_pointer_cast(d_data.configs.subparts_end.data());
    const u16 *d_subparts_classes_start = thrust::raw_pointer_cast(d_data.subparts.classes_start.data());
    const u16 *d_subparts_classes_end = thrust::raw_pointer_cast(d_data.subparts.classes_end.data());
    const u32 *d_class_limit = thrust::raw_pointer_cast(d_data.classes.limit.data());
    const u16 *d_class_parent = thrust::raw_pointer_cast(d_data.classes.parent.data());
    const u16 *d_class_subpart_idx = thrust::raw_pointer_cast(d_data.classes.subpart_idx.data());
    const parser::TimeSlots *d_time_opt_times = thrust::raw_pointer_cast(d_data.time_options.times.data());
    const u16 *d_student_course_idxs = thrust::raw_pointer_cast(d_data.students.course_idxs.data());
    const usize *d_student_course_offsets = thrust::raw_pointer_cast(d_data.students.course_idxs_offsets.data());

    usize n_students = d_data.students.id.size();
    constexpr dim3 block_dim(1024);
    dim3 grid_dim(static_cast<u32>(population.population_size));
    usize sh_mem_size = sizeof(usize) + (n_classes + MAX_SUBPARTS + 1) * sizeof(u16) + (n_classes + 4) * sizeof(bool);
    k_assign_students<<<grid_dim, block_dim, sh_mem_size>>>(
        d_students_idxs, d_class_counts, d_pop_times, d_courses_configs_start, d_courses_configs_end,
        d_configs_subparts_start, d_configs_subparts_end, d_subparts_classes_start, d_subparts_classes_end,
        d_class_limit, d_class_parent, d_class_subpart_idx, d_time_opt_times, d_student_course_idxs,
        d_student_course_offsets, n_classes, n_students);

    cudaErrCheck(cudaDeviceSynchronize());
}

}