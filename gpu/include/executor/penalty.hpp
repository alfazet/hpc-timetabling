#ifndef GPU_TIMETABLING_PENALTY_HPP
#define GPU_TIMETABLING_PENALTY_HPP

#include <iostream>

#include "typedefs.hpp"

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