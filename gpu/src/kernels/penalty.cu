#include "kernels/penalty.cuh"

Penalty::Penalty(u32 hard, u32 soft) : penalty(uint2(hard, soft)) {
}

std::ostream &operator<<(std::ostream &stream, const Penalty &p) {
    if (p.penalty.x > 0) {
        stream << "hard violations: " << p.penalty.x;
    } else {
        stream << "soft penalty: " << p.penalty.y;
    }

    return stream;
}