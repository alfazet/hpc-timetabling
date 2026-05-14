#ifndef GPU_TIMETABLING_PENALTY_CUH
#define GPU_TIMETABLING_PENALTY_CUH

#include <ostream>

#include "typedefs.hpp"

namespace kernels {

struct Penalty {
    u32 hard = 0;
    u32 soft = 0;

    Penalty() = default;

    __host__ __device__ Penalty(u32 hard, u32 soft) : hard(hard), soft(soft) {}

    __host__ __device__ bool operator<(const Penalty &p) const {
        if (hard == p.hard) {
            return soft < p.soft;
        }
        return hard < p.hard;
    }

    __host__ __device__ bool operator==(const Penalty &p) const { return soft == p.soft && hard == p.hard; }

    __host__ __device__ bool operator!=(const Penalty &p) const { return !(*this == p); }

    void print(std::ostream &out) const;
};

static Penalty MAX_PENALTY(UINT32_MAX, UINT32_MAX);

} // namespace kernels

#endif // GPU_TIMETABLING_PENALTY_CUH
