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

Penalty operator+(Penalty p1, Penalty p2);

#endif //GPU_TIMETABLING_PENALTY_CUH