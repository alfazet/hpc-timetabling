#include "kernels/selection.cuh"

namespace kernels {
Selection::Selection(usize population_size, f32 frac) : selected(std::ceil(frac * population_size) + 1), frac(frac) {}

void Selection::select(const Population &population) {
    // TODO
}

} // namespace kernels