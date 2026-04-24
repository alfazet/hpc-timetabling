#include "kernels/distribution.cuh"
#include <thrust/device_vector.h>

Distribution::Distribution(const Solution &sol, const TimetableData &data)
    : sol(sol), data(data) {}

Penalty Distribution::calculate_penalty() {
    auto constraints_count = this->data.distributions.size();
    constexpr auto threadsPerBlock = 64;

    thrust::device_vector<u32> device_penalties(constraints_count);

    kernels::distribution_penalty_calculation()

    return penalty;
}

void kernels::distribution_penalty_calculation(const Distribution *dist,
                                               u32 *penalty) {

}