#ifndef GPU_TIMETABLING_COMMON_CUH
#define GPU_TIMETABLING_COMMON_CUH

#include <cstdio>
#include <cstdlib>
#include <cuda_runtime.h>

#include "typedefs.hpp"

#define ERR_AND_DIE(reason) \
do { \
    fprintf(stderr, "fatal error in %s, line %d\n", __FILE__, __LINE__); \
    fprintf(stderr, "reason: %s\n", (reason)); \
    exit(EXIT_FAILURE); \
} while (0)

inline void cudaErrCheck(const cudaError_t res) {
    if (res != cudaSuccess)
        ERR_AND_DIE(cudaGetErrorString(res));
}

#endif //GPU_TIMETABLING_COMMON_CUH