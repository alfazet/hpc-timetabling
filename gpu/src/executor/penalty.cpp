#include "executor/penalty.hpp"

Penalty::Penalty(u32 packed) : hard((packed & HARD_PENALTY_MASK) >>
                                    SOFT_PENALTY_BITS),
                               soft(packed & SOFT_PENALTY_MASK) {
}

std::ostream &operator<<(std::ostream &stream, const Penalty &p) {
    if (p.hard > 0) {
        stream << "hard violations: " << p.hard;
    } else {
        stream << "soft penalty: " << p.soft;
    }

    return stream;
}

Penalty operator+(Penalty p1, const Penalty p2) {
    p1.soft += p2.soft;
    p1.hard += p2.hard;
    return p1;
}