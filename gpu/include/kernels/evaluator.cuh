#ifndef GPU_TIMETABLING_EVALUATOR_CUH
#define GPU_TIMETABLING_EVALUATOR_CUH

#include "assigner.cuh"
#include "population.cuh"

namespace kernels::evaluator {

// compute the penalty of all solutions in the population
// (one block per one solution)
void evaluate(const TimetableData &d_data, Population &population, const StudentAssignment &assignment);

}

#endif //GPU_TIMETABLING_EVALUATOR_CUH