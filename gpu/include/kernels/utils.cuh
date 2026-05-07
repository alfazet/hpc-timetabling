#ifndef GPU_TIMETABLING_UTILS_CUH
#define GPU_TIMETABLING_UTILS_CUH

#include "parser/timeslots.hpp"

namespace kernels::utils {

__device__ inline bool timeslots_overlap(const parser::TimeSlots &a, const parser::TimeSlots &b) {
    if ((a.weeks.bits & b.weeks.bits) == 0) {
        return false;
    }
    if ((a.days.bits & b.days.bits) == 0) {
        return false;
    }

    return a.start < b.start + b.length && b.start < a.start + a.length;
}

} // namespace kernels::utils

#endif // GPU_TIMETABLING_UTILS_CUH
