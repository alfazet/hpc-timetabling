#include "kernels/assigner.cuh"

namespace kernels {

__device__ bool timeslots_overlap(const parser::TimeSlots &a,
                                  const parser::TimeSlots &b) {
    if ((a.weeks.bits & b.weeks.bits) == 0) {
        return false;
    }
    if ((a.days.bits & b.days.bits) == 0) {
        return false;
    }
    return a.start < b.start + b.length && b.start < a.start + a.length;
}

__global__ void assign_students_kernel(
    u16 *students_idxs,
    u32 *class_counts,
    const u16 *pop_times,
    const u16 *courses_configs_start, const u16 *courses_configs_end,
    const u16 *configs_subparts_start, const u16 *configs_subparts_end,
    const u16 *subparts_classes_start, const u16 *subparts_classes_end,
    const u32 *class_limit, const u16 *class_parent,
    const u16 *class_subpart_idx,
    const parser::TimeSlots *time_opt_times,
    const u16 *student_course_idxs, const usize *student_course_offsets,
    usize n_classes, usize n_students) {
    usize sol = blockIdx.x;
    usize tid = threadIdx.x;

    extern __shared__ char sh_mem[];
    u32 *sh_class_count = reinterpret_cast<u32 *>(sh_mem);
    bool *sh_already_attending = reinterpret_cast<bool *>(
        sh_class_count + n_classes);
    for (usize i = tid; i < n_classes; i += blockDim.x) {
        sh_class_count[i] = 0;
    }
    __syncthreads();

    usize sol_offset = sol * n_classes;
    constexpr usize MAX_SUBPARTS = 64; // this is enough, right?
    // class index chosen per subpart in a given config
    usize local_assignment[MAX_SUBPARTS];

    for (usize student_idx = 0; student_idx < n_students; student_idx++) {
        for (usize i = tid; i < n_classes; i += blockDim.x) {
            sh_already_attending[i] = false;
        }
        __syncthreads();

        if (tid == 0) {
            for (usize i = 0; i < n_classes; i++) {
                u32 cnt = sh_class_count[i];
                usize offset = MAX_CLASS_LIMIT * (sol * n_classes + i);
                for (u32 j = 0; j < cnt; j++) {
                    if (students_idxs[offset + j] == student_idx) {
                        sh_already_attending[i] = true;
                        break;
                    }
                }
            }
        }
        __syncthreads();

        usize courses_start = student_course_offsets[student_idx];
        usize courses_end = student_course_offsets[student_idx + 1];
        for (usize i = courses_start; i < courses_end; ++i) {
            usize course_idx = student_course_idxs[i];
            usize configs_start = courses_configs_start[course_idx];
            usize configs_end = courses_configs_end[course_idx];
            bool course_assigned = false;

            for (usize config_idx = configs_start; config_idx < configs_end && !course_assigned; config_idx++) {
                // assigning is inherently sequential (due to parents), so it's done in only one thread per block
                if (tid == 0) {
                    usize subparts_start = configs_subparts_start[config_idx];
                    usize subparts_end = configs_subparts_end[config_idx];
                    usize n_subparts = subparts_end - subparts_start;
                    for (usize j = 0; j < n_subparts && j < MAX_SUBPARTS; j++) {
                        local_assignment[j] = NO_CLASS_ASSIGNED;
                    }
                    bool config_ok = true;
                    for (usize subpart_idx = subparts_start; subpart_idx < subparts_end && config_ok; subpart_idx++) {
                        usize classes_start = subparts_classes_start[subpart_idx];
                        usize classes_end = subparts_classes_end[subpart_idx];
                        bool subpart_assigned = false;

                        for (usize class_idx = classes_start; class_idx < classes_end && !subpart_assigned; class_idx
                             ++) {
                            usize trial[MAX_SUBPARTS];
                            for (usize j = 0; j < n_subparts; ++j) {
                                trial[j] = local_assignment[j];
                            }
                            usize cur = class_idx;
                            // resolve parents
                            while (true) {
                                usize subpart_offset = class_subpart_idx[cur] - subparts_start;
                                if (subpart_offset < MAX_SUBPARTS) {
                                    trial[subpart_offset] = cur;
                                }
                                usize par = class_parent[cur];
                                if (par == NO_PARENT) {
                                    break;
                                }
                                cur = par;
                            }

                            // check limits
                            bool ok = true;
                            for (usize j = 0; j < n_subparts && ok; j++) {
                                if (trial[j] == NO_CLASS_ASSIGNED || sh_already_attending[trial[j]]) {
                                    continue;
                                }
                                u32 limit = class_limit[trial[j]];
                                if (limit != NO_LIMIT && sh_class_count[trial[j]] + 1 > limit) {
                                    ok = false;
                                }
                            }

                            // check time confilicts
                            if (ok) {
                                for (usize j = 0; j < n_subparts && ok; j++) {
                                    if (trial[j] == NO_CLASS_ASSIGNED || sh_already_attending[trial[j]]) {
                                        continue;
                                    }
                                    usize t_opt = pop_times[sol_offset + trial[j]];
                                    const parser::TimeSlots &new_time = time_opt_times[t_opt];
                                    // check against all classes the student already attends
                                    for (usize k = 0; k < n_classes && ok; k++) {
                                        if (!sh_already_attending[k]) {
                                            continue;
                                        }
                                        usize at_opt = pop_times[sol_offset + k];
                                        const parser::TimeSlots &at_time = time_opt_times[at_opt];
                                        if (timeslots_overlap(new_time, at_time)) {
                                            ok = false;
                                        }
                                    }
                                    // also check against other trial classes
                                    for (usize k = 0; k < j && ok; k++) {
                                        if (trial[k] == NO_CLASS_ASSIGNED || sh_already_attending[trial[k]]) {
                                            continue;
                                        }
                                        usize t = pop_times[sol_offset + trial[k]];
                                        const parser::TimeSlots &t_time = time_opt_times[t];
                                        if (timeslots_overlap(new_time, t_time)) {
                                            ok = false;
                                        }
                                    }
                                }
                            }
                            if (ok) {
                                for (usize j = 0; j < n_subparts; j++) {
                                    local_assignment[j] = trial[j];
                                }
                                subpart_assigned = true;
                            }
                        }
                        if (!subpart_assigned) {
                            config_ok = false;
                        }
                    }
                    if (config_ok) {
                        // commit the assignment
                        for (usize j = 0; j < n_subparts; j++) {
                            usize c = local_assignment[j];
                            if (c == NO_CLASS_ASSIGNED || sh_already_attending[c]) {
                                continue;
                            }
                            u32 cnt = sh_class_count[c];
                            usize offset = MAX_CLASS_LIMIT * (sol * n_classes + c);
                            students_idxs[offset + cnt] = student_idx;
                            sh_class_count[c] = cnt + 1;
                            sh_already_attending[c] = true;
                        }
                        course_assigned = true;
                    }
                }
                __syncthreads();
            }
        }
        __syncthreads();
    }
    for (usize i = tid; i < n_classes; i += blockDim.x) {
        class_counts[sol_offset + i] = sh_class_count[i];
    }
}

StudentAssignment::StudentAssignment(usize n_classes, usize population_size)
    : students_idxs(n_classes * population_size * MAX_CLASS_LIMIT),
      class_counts(n_classes * population_size), n_classes(n_classes),
      population_size(population_size) {
}

void StudentAssignment::assign(const TimetableData &d_data,
                               const Population &population) {
    thrust::fill(students_idxs.begin(), students_idxs.end(), 0);
    thrust::fill(class_counts.begin(), class_counts.end(), 0);

    u16 *d_students_idxs =
        thrust::raw_pointer_cast(this->students_idxs.data());
    u32 *d_class_counts = thrust::raw_pointer_cast(this->class_counts.data());

    const u16 *d_pop_times =
        thrust::raw_pointer_cast(population.times.data());
    const u16 *d_courses_configs_start =
        thrust::raw_pointer_cast(d_data.courses.configs_start.data());
    const u16 *d_courses_configs_end =
        thrust::raw_pointer_cast(d_data.courses.configs_end.data());
    const u16 *d_configs_subparts_start =
        thrust::raw_pointer_cast(d_data.configs.subparts_start.data());
    const u16 *d_configs_subparts_end =
        thrust::raw_pointer_cast(d_data.configs.subparts_end.data());
    const u16 *d_subparts_classes_start =
        thrust::raw_pointer_cast(d_data.subparts.classes_start.data());
    const u16 *d_subparts_classes_end =
        thrust::raw_pointer_cast(d_data.subparts.classes_end.data());
    const u32 *d_class_limit =
        thrust::raw_pointer_cast(d_data.classes.limit.data());
    const u16 *d_class_parent =
        thrust::raw_pointer_cast(d_data.classes.parent.data());
    const u16 *d_class_subpart_idx =
        thrust::raw_pointer_cast(d_data.classes.subpart_idx.data());
    const parser::TimeSlots *d_time_opt_times =
        thrust::raw_pointer_cast(d_data.time_options.times.data());
    const u16 *d_student_course_idxs =
        thrust::raw_pointer_cast(d_data.students.course_idxs.data());
    const usize *d_student_course_offsets =
        thrust::raw_pointer_cast(d_data.students.course_idxs_offsets.data());

    usize n_students = d_data.students.id.size();
    constexpr dim3 block_dim(1024);
    dim3 grid_dim(static_cast<u32>(population.population_size));
    usize sh_mem_size =
        n_classes * sizeof(u32) + n_classes * sizeof(bool);
    assign_students_kernel<<<grid_dim, block_dim, sh_mem_size>>>(
        d_students_idxs, d_class_counts, d_pop_times, d_courses_configs_start,
        d_courses_configs_end, d_configs_subparts_start, d_configs_subparts_end, d_subparts_classes_start,
        d_subparts_classes_end,
        d_class_limit, d_class_parent, d_class_subpart_idx, d_time_opt_times,
        d_student_course_idxs, d_student_course_offsets, n_classes, n_students);

    cudaErrCheck(cudaDeviceSynchronize());
}

}