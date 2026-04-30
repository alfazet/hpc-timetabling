#include <stdexcept>

#include "executor/cmd_args.hpp"

// list all optional arguments here
std::unordered_map<std::string, void(ArgParser::*)(ArgsList &) const>
ArgParser::flag_parsers{
    {"-g", &ArgParser::parse_generations},
    {"-p", &ArgParser::parse_population_size},
    {"-s", &ArgParser::parse_seed},
    {"-o", &ArgParser::parse_output_path},
};

u32 parse_u32(const char *s, const char *arg_name) {
    char *end = nullptr;
    u32 x = std::strtoul(s, &end, 10);
    if (*end != '\0') {
        throw std::runtime_error(
            "expected a uint value for " + std::string(arg_name));
    }

    return x;
}

f32 parse_f32(const char *s, const char *arg_name) {
    char *end = nullptr;
    f32 x = std::strtof(s, &end);
    if (*end != '\0') {
        throw std::runtime_error(
            "expected a float value for " + std::string(arg_name));
    }

    return x;
}

ArgParser::ArgParser(usize n_args_, char **values_) : n_args(n_args_),
    values(values_) {
}

/// assumes that CLI args are <flag_1> <value_1> <flag_2> <value_2> ...
/// the first argument should always be the dataset path
ArgsList ArgParser::parse_all() {
    ArgsList list{};
    if (n_args < 1) {
        display_help();
        throw std::runtime_error("dataset file path is required");
    }
    list.dataset_path = this->values[0];

    this->arg_i = 1;
    while (this->arg_i < this->n_args) {
        const char *flag = this->values[this->arg_i];
        auto iter = flag_parsers.find(flag);
        if (iter == flag_parsers.end()) {
            display_help();
            throw std::runtime_error("invalid flag " + std::string(flag));
        }
        const auto &parser = iter->second;

        // move to the flag's value and parse it
        this->arg_i++;
        (this->*parser)(list);
        // move to the next flag
        this->arg_i++;
    }

    return list;
}

void ArgParser::parse_generations(ArgsList &list) const {
    if (this->arg_i >= this->n_args) {
        display_help();
        throw std::runtime_error("missing value for generations");
    }
    list.generations = parse_u32(this->values[this->arg_i],
                                 "generations");
}

void ArgParser::parse_population_size(ArgsList &list) const {
    if (this->arg_i >= this->n_args) {
        display_help();
        throw std::runtime_error("missing value for population_size");
    }
    list.population_size = parse_u32(this->values[this->arg_i],
                                     "population_size");
}

void ArgParser::parse_seed(ArgsList &list) const {
    if (this->arg_i >= this->n_args) {
        display_help();
        throw std::runtime_error("missing value for seed");
    }
    list.seed = parse_u32(this->values[this->arg_i], "seed");
}

void ArgParser::parse_output_path(ArgsList &list) const {
    if (this->arg_i >= this->n_args) {
        display_help();
        throw std::runtime_error("missing value for output path");
    }
    list.output_path = this->values[this->arg_i];
}

void ArgParser::display_help() {
    printf(
        "Arguments:\n"
        "<dataset_path> [-g] [-p]\n"
        "where:\n"
        "- `-g` = number of generations\n"
        "- `-p` = population size\n"
        "\n"
        );
}