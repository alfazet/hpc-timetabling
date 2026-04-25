#include "parser/parser.hpp"
#include "executor/cmd_args.hpp"
#include "kernels/model.cuh"

void main_(int argc, char **argv) {
    ArgParser arg_parser(argc - 1, argv + 1);
    auto arg_list = arg_parser.parse_all();
    auto content = parser::utils::read_file(arg_list.dataset_path);
    auto problem = parser::Problem::parse(content);
    auto data = kernels::TimetableData::from_problem(problem);
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