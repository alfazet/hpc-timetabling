#include <charconv>
#include <cstring>
#include <stdexcept>

#include "executor/cmd_args.hpp"

// list all optional arguments in `include/executor/cmd_args.hpp`

u32 parse_u32(const char *s, const char *arg_name) {
    u32 value;
    auto [ptr, ec] = std::from_chars(s, s + std::strlen(s), value);

    if (ec != std::errc() || *ptr != '\0') {
        throw std::runtime_error("invalid uint value for " + std::string(arg_name));
    }

    return value;
}

f32 parse_f32(const char *s, const char *arg_name) {
    char *end = nullptr;
    f32 x = std::strtof(s, &end);
    if (*end != '\0') {
        throw std::runtime_error("expected a float value for " + std::string(arg_name));
    }

    return x;
}

std::string parse_string(const char *s, const char *arg_name) {
    if (!s || *s == '\0') {
        throw std::runtime_error("expected a string value for " + std::string(arg_name));
    }

    return s;
}

using ParserFn = void (ArgParser::*)(ArgsList &) const;
std::unordered_map<std::string, ParserFn> ArgParser::flag_parsers = {
#define X(flag, field, type, parser, default_val, help) {flag, &ArgParser::parse_##field},
    ARG_TABLE(X)
#undef X
};

#define X(flag, field, type, parser, default_val, help)                                                                \
    void ArgParser::parse_##field(ArgsList &list) const {                                                              \
        if (this->arg_i >= this->n_args) {                                                                             \
            display_help();                                                                                            \
            throw std::runtime_error("missing value for " #field);                                                     \
        }                                                                                                              \
        list.field = parser(this->values[this->arg_i], #field);                                                        \
    }
ARG_TABLE(X)
#undef X

void ArgParser::display_help() {
    printf("Arguments:\n<dataset_path> [flags]\nwhere:\n");
#define X(flag, field, type, parser, default_val, help) printf("  %s : %s\n", flag, help);
    ARG_TABLE(X)
#undef X
}

ArgParser::ArgParser(usize n_args_, char **values_) : n_args(n_args_), values(values_) {}

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
