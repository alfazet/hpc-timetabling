#include "parser/parser.h"
#include "serializer/serializer.h"
#include "executor/cmd_args.h"

using parser::Problem;
using serializer::OutputMetadata;

void main_(int argc, char **argv) {
    ArgParser arg_parser(argc - 1, argv + 1);
    auto arg_list = arg_parser.parse_all();

    printf("generations: %u, population_size: %u\n", arg_list.generations,
           arg_list.population_size);
}

int main(int argc, char **argv) {
    try {
        main_(argc, argv);
    } catch (std::exception &e) {
        printf("error: %s\n", e.what());
        return 1;
    }

    return 0;
}