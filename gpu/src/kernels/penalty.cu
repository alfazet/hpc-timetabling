#include "kernels/penalty.cuh"
#include <stdio.h>

namespace kernels {

void Penalty::print() const {
    if (this->hard != 0)
        printf("hard: %u, ", this->hard);
    printf("soft: %u", this->soft);
}

}
