#include "common.cuh"
#include "parser/parser.h"
#include "serializer/serializer.h"

using parser::Problem;
using serializer::OutputMetadata;

int main(int argc, char **argv) {
    if (argc < 2) {
        ERR_AND_DIE("xml data not provided");
    }
    std::string path(argv[1]);
    auto xml = parser::utils::read_file(path);
    auto problem = Problem::parse(xml);

    auto metadata = OutputMetadata::from_problem(problem);
    printf("problem name: %s\n", metadata.name.c_str());
    printf("technique: %s\n", metadata.technique.c_str());

    return 0;
}