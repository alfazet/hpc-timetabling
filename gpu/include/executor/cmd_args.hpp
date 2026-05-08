#ifndef GPU_TIMETABLING_CMD_ARGS_H
#define GPU_TIMETABLING_CMD_ARGS_H

#include <string>
#include <unordered_map>

#include "typedefs.hpp"

constexpr u32 DEFAULT_GENERATIONS = 512;
constexpr u32 DEFAULT_POPULATION_SIZE = 2048;
constexpr u32 DEFAULT_SEED = 21372137;
constexpr f32 DEFAULT_SEL_FRAC = 0.25;
constexpr f32 DEFAULT_CROSSOVER_RATE = 0.9;
constexpr f32 DEFAULT_MUTATION_RATE = 0.1;
constexpr u32 DEFAULT_MUTATION_TRIALS = 8;
constexpr f32 DEFAULT_ELITES_FRAC = 0.05;
constexpr u32 DEFAULT_LS_ITERS = 8;
constexpr u32 DEFAULT_LS_TRIALS = 8;
constexpr f32 DEFAULT_REPLACEMENT_RATE = 0.1;
constexpr std::string DEFAULT_OUTPUT_PATH = "./solution.xml";

// list all optional arguments here
#define ARG_TABLE(X)                                                                                                   \
    X("-g", generations, u32, parse_u32, DEFAULT_GENERATIONS, "number of generations")                                 \
    X("-p", population_size, u32, parse_u32, DEFAULT_POPULATION_SIZE, "population size")                               \
    X("--sel-frac", sel_frac, f32, parse_f32, DEFAULT_SEL_FRAC, "fraction of population size to select for crossover") \
    X("--cross", cross_rate, f32, parse_f32, DEFAULT_CROSSOVER_RATE, "crossover rate")                                 \
    X("--mut-rate", mut_rate, f32, parse_f32, DEFAULT_MUTATION_RATE, "mutation rate")                                  \
    X("--mut-trials", mut_trials, u32, parse_u32, DEFAULT_MUTATION_TRIALS, "mutation trials per iteration")            \
    X("--elit-frac", elites_frac, f32, parse_f32, DEFAULT_ELITES_FRAC, "fraction of population to keep as elite")      \
    X("--ls-iters", ls_iters, u32, parse_u32, DEFAULT_LS_ITERS, "local search iterations per generation")              \
    X("--ls-trials", ls_trials, u32, parse_u32, DEFAULT_LS_TRIALS, "local search trials per iteration")                \
    X("--repl-rate", repl_rate, f32, parse_f32, DEFAULT_REPLACEMENT_RATE,                                              \
      "fraction of population to replace with new random solutions")                                                   \
    X("-s", seed, u32, parse_u32, DEFAULT_SEED, "random seed")                                                         \
    X("-o", output_path, std::string, parse_string, DEFAULT_OUTPUT_PATH, "solution output path")

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
