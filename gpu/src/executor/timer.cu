#include "executor/timer.cuh"
#include "kernels/common.cuh"

void Timer::start() {
    this->elapsed = 0;
    cudaErrCheck(cudaEventCreate(&start_event));
    cudaErrCheck(cudaEventCreate(&stop_event));
    cudaErrCheck(cudaEventRecord(start_event));
}

void Timer::stop() {
    cudaErrCheck(cudaEventRecord(stop_event));
    cudaErrCheck(cudaEventSynchronize(stop_event));
    float elapsed;
    cudaErrCheck(cudaEventElapsedTime(&elapsed, start_event, stop_event));
    this->elapsed = elapsed;
    cudaErrCheck(cudaEventDestroy(start_event));
    cudaErrCheck(cudaEventDestroy(stop_event));
}

void Timer::print(u32 generations) {
    printf("Average time per generation (over the last %u): %.4f ms\n\n", generations, elapsed / generations);
}