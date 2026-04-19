#include "parser/parser.h"
#include "serializer/serializer.h"
#include "executor/cmd_args.h"

using parser::Problem;
using serializer::OutputMetadata;

int main(int argc, char **argv) {
    ArgParser arg_parser(argc - 1, argv + 1);
    auto arg_list = arg_parser.parse_all();

    return 0;
}