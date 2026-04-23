#ifndef GPU_TIMETABLING_CMD_ARGS_H
#define GPU_TIMETABLING_CMD_ARGS_H

#include <string>
#include <unordered_map>

#include "typedefs.hpp"

constexpr u32 DEFAULT_GENERATIONS = 600;
constexpr u32 DEFAULT_POPULATION_SIZE = 24000;

struct ArgsList {
    std::string dataset_path;
    u32 generations = DEFAULT_GENERATIONS;
    u32 population_size = DEFAULT_POPULATION_SIZE;
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
};

#endif //GPU_TIMETABLING_CMD_ARGS_H