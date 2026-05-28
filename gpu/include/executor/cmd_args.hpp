#ifndef GPU_TIMETABLING_CMD_ARGS_H
#define GPU_TIMETABLING_CMD_ARGS_H

#include <ostream>
#include <string>
#include <unordered_map>

#include "typedefs.hpp"

constexpr u32 DEFAULT_GENERATIONS = 128;
constexpr u32 DEFAULT_POPULATION_SIZE = 512;
constexpr u32 DEFAULT_SEED = 21372137;
constexpr f32 DEFAULT_SEL_FRAC = 0.2;
constexpr f32 DEFAULT_CROSSOVER_RATE_MIN = 0.5;
constexpr f32 DEFAULT_CROSSOVER_RATE_MAX = 0.9;
constexpr f32 DEFAULT_MUTATION_RATE_MIN = 0.1;
constexpr f32 DEFAULT_MUTATION_RATE_MAX = 0.5;
constexpr u32 DEFAULT_MUTATION_TRIALS = 64;
constexpr f32 DEFAULT_ELITES_FRAC_MIN = 0.025;
constexpr f32 DEFAULT_ELITES_FRAC_MAX = 0.075;
constexpr f32 DEFAULT_WORST_FRAC_MIN = 0.05;
constexpr f32 DEFAULT_WORST_FRAC_MAX = 0.1;
constexpr u32 DEFAULT_LS_ITERS = 128;
constexpr u32 DEFAULT_TOURNAMENT_SIZE = 4;
constexpr std::string DEFAULT_OUTPUT_PATH = "./solution.xml";

// list all optional arguments here
#define ARG_TABLE(X)                                                                                                   \
    X("-g", generations, u32, parse_u32, DEFAULT_GENERATIONS, "number of generations")                                 \
    X("-p", population_size, u32, parse_u32, DEFAULT_POPULATION_SIZE, "population size")                               \
    X("--sel-frac", sel_frac, f32, parse_f32, DEFAULT_SEL_FRAC, "fraction of population size to select for crossover") \
    X("--cross-min", cross_rate_min, f32, parse_f32, DEFAULT_CROSSOVER_RATE_MIN, "min crossover rate")                 \
    X("--cross-max", cross_rate_max, f32, parse_f32, DEFAULT_CROSSOVER_RATE_MAX, "max crossover rate")                 \
    X("--mut-rate-min", mut_rate_min, f32, parse_f32, DEFAULT_MUTATION_RATE_MIN, "min mutation rate")                  \
    X("--mut-rate-max", mut_rate_max, f32, parse_f32, DEFAULT_MUTATION_RATE_MAX, "max mutation rate")                  \
    X("--mut-trials", mut_trials, u32, parse_u32, DEFAULT_MUTATION_TRIALS, "mutation trials per iteration")            \
    X("--elit-frac-min", elites_frac_min, f32, parse_f32, DEFAULT_ELITES_FRAC_MIN,                                     \
      "min fraction of population to keep as elite")                                                                   \
    X("--elit-frac-max", elites_frac_max, f32, parse_f32, DEFAULT_ELITES_FRAC_MAX,                                     \
      "max fraction of population to keep as elite")                                                                   \
    X("--worst-frac-min", worst_frac_min, f32, parse_f32, DEFAULT_WORST_FRAC_MIN,                                      \
      "min fraction of the worst solutions to replace")                                                                \
    X("--worst-frac-max", worst_frac_max, f32, parse_f32, DEFAULT_WORST_FRAC_MAX,                                      \
      "max fraction of the worst solutions to replace")                                                                \
    X("--ls-iters", ls_iters, u32, parse_u32, DEFAULT_LS_ITERS, "local search iterations per generation")              \
    X("--tour-size", tournament_size, u32, parse_u32, DEFAULT_TOURNAMENT_SIZE,                                         \
      "tournament selection size (must be 2, 4, 8, 16 or 32)")                                                         \
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
    std::ostream &out;

    ArgParser(usize n_args_, char **values_, std::ostream &out_);

    ArgsList parse_all();

    static void display_help(std::ostream &out);

  private:
    static std::unordered_map<std::string, void (ArgParser::*)(ArgsList &) const> flag_parsers;

#define X(flag, field, type, parser, default_val, help) void parse_##field(ArgsList &list) const;
    ARG_TABLE(X)
#undef X
};

#endif // GPU_TIMETABLING_CMD_ARGS_H
