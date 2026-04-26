#include "kernels/penalty.cuh"

namespace kernels {

Penalty::Penalty(u32 hard, u32 soft) : penalty(uint2(hard, soft)) {
}

}