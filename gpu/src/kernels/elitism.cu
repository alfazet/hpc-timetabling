#include "kernels/elitism.cuh"

namespace kernels {

Elitism::Elitism(usize population_size, f32 frac) : elites(std::ceil(population_size * frac) + 1), frac(frac) {}

void Elitism::choose_elites(const Population &population) {
    // TODO
}

} // namespace kernels