#include "common.cuh"
#include "parser/parser.h"

__global__ void vectorAdd(const float *a, const float *b, float *c, int n) {
    int id = blockDim.x * blockIdx.x + threadIdx.x;
    if (id < n) {
        c[id] = a[id] + b[id];
    }
}

int main() {
    int n = 100'000;

    std::vector<float> h_a(n), h_b(n), h_c(n);
    for (int i = 0; i < n; i++) {
        h_a[i] = rand() / (float)RAND_MAX;
        h_b[i] = rand() / (float)RAND_MAX;
    }

    float *d_a, *d_b, *d_c;
    cudaErrCheck(cudaMalloc((void **)&d_a, n * sizeof(float)));
    cudaErrCheck(cudaMalloc((void **)&d_b, n * sizeof(float)));
    cudaErrCheck(cudaMalloc((void **)&d_c, n * sizeof(float)));
    cudaErrCheck(cudaMemcpy(d_a, h_a.data(), n * sizeof(float),
                            cudaMemcpyHostToDevice));
    cudaErrCheck(cudaMemcpy(d_b, h_b.data(), n * sizeof(float),
                            cudaMemcpyHostToDevice));

    int blockDim = 1024;
    int gridDim = (n + blockDim - 1) / blockDim;
    vectorAdd<<<gridDim, blockDim>>>(d_a, d_b, d_c, n);
    cudaErrCheck(cudaMemcpy(h_c.data(), d_c, n * sizeof(float),
                            cudaMemcpyDeviceToHost));

    for (int i = 0; i < n; i++) {
        if (abs(h_a[i] + h_b[i] - h_c[i]) > 1e-9) {
            ERR_AND_DIE("vectorAdd failed");
        }
    }

    cudaErrCheck(cudaFree(d_a));
    cudaErrCheck(cudaFree(d_b));
    cudaErrCheck(cudaFree(d_c));
    printf("cuda test finished successfully\n");

    std::string xml = parser::utils::read_file(
        "../../../data/itc2019/sample.xml");
    auto problem = parser::Problem::parse(xml);
    printf("problem name: %s\n", problem.name.c_str());
    printf("#students in sample: %zu\n", problem.students.items.size());

    return 0;
}