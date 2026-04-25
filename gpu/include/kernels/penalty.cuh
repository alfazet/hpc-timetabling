#ifndef GPU_TIMETABLING_PENALTY_CUH
#define GPU_TIMETABLING_PENALTY_CUH

#include <iostream>

#include "typedefs.hpp"
#include "common.cuh"

struct Penalty {
    uint2 penalty;

    explicit Penalty(u32 hard, u32 soft);
};

std::ostream &operator<<(std::ostream &stream, const Penalty &p);

#endif //GPU_TIMETABLING_PENALTY_CUH