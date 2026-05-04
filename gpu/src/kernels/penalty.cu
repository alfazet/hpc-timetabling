#include <cstdio>

#include "kernels/penalty.cuh"

namespace kernels {

void Penalty::print() const {
    if (this->hard != 0) {
        printf("hard: %u, ", this->hard);
    }
    printf("soft: %u", this->soft);
}

} // namespace kernels
