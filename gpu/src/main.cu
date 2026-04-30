#include "parser/parser.hpp"
#include "executor/cmd_args.hpp"
#include "executor/solver.cuh"
#include "kernels/model.cuh"
#include "serializer/serializer.hpp"

void main_(int argc, char **argv) {
    ArgParser arg_parser(argc - 1, argv + 1);
    auto arg_list = arg_parser.parse_all();
    auto content = parser::utils::read_file(arg_list.dataset_path);
    auto problem = parser::Problem::parse(content);
    auto d_data = kernels::TimetableData::from_problem(problem);
    auto metadata = serializer::OutputMetadata::from_problem(problem);

    Solver solver(d_data, arg_list.generations, arg_list.population_size,
                  arg_list.seed);
    auto best_solution = solver.solve();
    auto output = best_solution.serialize(d_data);
    auto xml = output.serialize(metadata);
    serializer::utils::write_file(arg_list.output_path, xml);
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