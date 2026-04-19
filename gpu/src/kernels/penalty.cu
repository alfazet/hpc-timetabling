#include <cstdio>

#include "kernels/common.cuh"
#include "kernels/penalty.cuh"

namespace kernels {
void foo() {
    printf("bar: %hu\n", BAR);
    i32 cuda_dev_count;
    cudaGetDeviceCount(&cuda_dev_count);
    printf("number of detected cuda devices: %d\n", cuda_dev_count);

    ERR_AND_DIE("dead");
}

}