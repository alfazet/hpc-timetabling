#include "kernels/distribution.cuh"

Distribution::Distribution(const Solution *sol, const TimetableData *data)
    : sol(sol), data(data) {}

u32 Distribution::calculate_penalty() const {
    const auto constraints_count = this->data->distributions.size();
    constexpr auto threadsPerBlock = 64;
    const auto blockCount = constraints_count / threadsPerBlock + 1;

    thrust::device_vector<u32> device_penalties(constraints_count);

    // Distributions in xml files are ordered, so the threads shouldn't
    // wait for each other :D
    kernels::distribution_penalty_calculation<<<blockCount, threadsPerBlock>>>(
        this->sol, this->data, device_penalties.data().get(), constraints_count);

    return thrust::reduce(device_penalties.begin(), device_penalties.end());
}

__global__ void kernels::distribution_penalty_calculation(const Solution *sol,
    const TimetableData *data, u32 *penal, const usize constraints_count) {

    const auto thread_idx = threadIdx.x + blockDim.x * blockIdx.x;
    if (thread_idx >= constraints_count)
        return;

    switch (data->distributions[thread_idx].kind.index()) {
    case 0:

        break;

    }
}