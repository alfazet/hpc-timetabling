#ifndef GPU_TIMETABLING_ASSIGNER_CUH
#define GPU_TIMETABLING_ASSIGNER_CUH

#include "common.cuh"
#include "model.cuh"
#include "population.cuh"

constexpr usize MAX_CLASS_LIMIT = 2048;

namespace kernels {

// TODO: this will take up a ton of memory for larger populations
// ideas:
// - replace usize indices with u16 (4x less space, and all indices in every problem are below 2^16 anyway)
// - ???
struct StudentAssignment {
    // (for class `i` in solution `j`)
    // indices of students taking this class are placed starting at
    // student_idxs[MAX_CLASS_LIMIT * (j * n_classes + i)] ...
    thrust::device_vector<usize> students_idxs;
    // and their count is class_counts[i * n_classes + j]
    thrust::device_vector<u32> class_counts;
    usize n_classes;
    usize population_size;

    StudentAssignment(usize n_classes, usize population_size);

    // find student assignments for solutions
    // (one block per one solution)
    void assign(const TimetableData &d_data, const Population &population);
};

}

#endif // GPU_TIMETABLING_ASSIGNER_CUH