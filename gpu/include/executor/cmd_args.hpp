#ifndef GPU_TIMETABLING_CMD_ARGS_H
#define GPU_TIMETABLING_CMD_ARGS_H

#include <string>
#include <unordered_map>

#include "typedefs.hpp"

constexpr u32 DEFAULT_GENERATIONS = 600;
constexpr u32 DEFAULT_POPULATION_SIZE = 24000;
constexpr u32 DEFAULT_SEED = 21372137;
constexpr std::string DEFAULT_OUTPUT_PATH = "./solution.xml";

struct ArgsList {
    std::string dataset_path;
    std::string output_path = DEFAULT_OUTPUT_PATH;
    u32 generations = DEFAULT_GENERATIONS;
    u32 population_size = DEFAULT_POPULATION_SIZE;
    u32 seed = DEFAULT_SEED;
};

class ArgParser {
public:
    usize n_args;
    char **values;
    usize arg_i = 0;

    ArgParser(usize n_args_, char **values_);

    ArgsList parse_all();

    static void display_help();

private:
    static std::unordered_map<std::string, void (
                                  ArgParser::*)(ArgsList &) const>
    flag_parsers;

    void parse_generations(ArgsList &list) const;

    void parse_population_size(ArgsList &list) const;

    void parse_seed(ArgsList &list) const;

    void parse_output_path(ArgsList &list) const;
};

#endif //GPU_TIMETABLING_CMD_ARGS_H