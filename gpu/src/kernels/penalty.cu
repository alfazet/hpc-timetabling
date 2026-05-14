#include "kernels/penalty.cuh"

namespace kernels {

void Penalty::print(std::ostream &out) const {
    if (this->hard != 0) {
        out << "hard: " << this->hard << ", ";
    }
    out << "soft: " << this->soft;
}

} // namespace kernels
