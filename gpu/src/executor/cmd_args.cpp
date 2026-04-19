#include <stdexcept>

#include "executor/cmd_args.h"

// list all optional arguments here
std::unordered_map<std::string, void (ArgParser::*)(ArgsList &)>
ArgParser::flag_parsers{
    {"-g", &ArgParser::parse_generations},
    {"-p", &ArgParser::parse_population_size},
};

ArgParser::ArgParser(int n_args_, char **values_) : n_args(n_args_),
                                                    values(values_) {
}

ArgsList ArgParser::parse_all() {
    ArgsList list{};
    if (n_args < 1) {
        display_help();
        throw std::runtime_error("dataset file path is required");
    }

    return list;
}

void ArgParser::parse_generations(ArgsList &args_list) {

}

void ArgParser::parse_population_size(ArgsList &args_list) {

}

void ArgParser::display_help() {
    printf("help\n");
}