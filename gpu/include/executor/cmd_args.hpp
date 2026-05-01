#ifndef GPU_TIMETABLING_CMD_ARGS_H
#define GPU_TIMETABLING_CMD_ARGS_H

#include <string>
#include <unordered_map>

#include "typedefs.hpp"

constexpr u32 DEFAULT_GENERATIONS = 600;
constexpr u32 DEFAULT_POPULATION_SIZE = 1000;
constexpr u32 DEFAULT_SEED = 21372137;
constexpr f32 DEFAULT_SEL_FRAC = 0.02;
constexpr std::string DEFAULT_OUTPUT_PATH = "./solution.xml";

// list all optional arguments here
#define ARG_TABLE(X)                                                                                                   \
    X("-g", generations, u32, parse_u32, DEFAULT_GENERATIONS, "number of generations")                                 \
    X("-p", population_size, u32, parse_u32, DEFAULT_POPULATION_SIZE, "population size")                               \
    X("-s", seed, u32, parse_u32, DEFAULT_SEED, "random seed")                                                         \
    X("-o", output_path, std::string, parse_string, DEFAULT_OUTPUT_PATH, "solution output path")                       \
    X("--sel-frac", sel_frac, f32, parse_f32, DEFAULT_SEL_FRAC,                                                        \
      "number of solutions to select for crossover (as a fraction of population size)")

struct ArgsList {
    std::string dataset_path;
#define X(flag, field, type, parser, default_val, help) type field = default_val;
    ARG_TABLE(X)
#undef X
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
    static std::unordered_map<std::string, void (ArgParser::*)(ArgsList &) const> flag_parsers;

#define X(flag, field, type, parser, default_val, help) void parse_##field(ArgsList &list) const;
    ARG_TABLE(X)
#undef X
};

#endif // GPU_TIMETABLING_CMD_ARGS_H
