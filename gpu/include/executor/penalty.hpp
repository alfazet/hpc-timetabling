#ifndef GPU_TIMETABLING_PENALTY_HPP
#define GPU_TIMETABLING_PENALTY_HPP

#include <iostream>

#include "typedefs.hpp"

// TODO:
// On the GPU represent the penalty in a single u32,
// with the hard penalty encoded on the more significant bits
// so that any hard penalty is worse than even the max soft penalty.
constexpr u32 SOFT_PENALTY_BITS = 16;
constexpr u32 SOFT_PENALTY_MASK = (1 << SOFT_PENALTY_BITS) - 1;
constexpr u32 HARD_PENALTY_MASK = UINT32_MAX - SOFT_PENALTY_MASK;

struct Penalty {
    u16 hard;
    u16 soft;

    explicit Penalty(u32 packed);
};

std::ostream &operator<<(std::ostream &stream, const Penalty &p);

#endif //GPU_TIMETABLING_PENALTY_HPP