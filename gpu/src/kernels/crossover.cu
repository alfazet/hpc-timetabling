#include "kernels/crossover.cuh"

namespace kernels {

Crossover::Crossover(f32 prob) : prob(prob) {}

void Crossover::next_population(const Selection &selection, Population &population, const Elitism &elitism) {}

} // namespace kernels