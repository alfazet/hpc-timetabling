#include "parser/parse_error.h"
#include "parser/days.h"

namespace parser {
Days Days::parse(const std::string &s) {
    u8 value = 0;
    constexpr usize max_days_per_week = 7;
    for (usize i = 0; i < s.size(); i++) {
        if (i >= max_days_per_week) {
            throw ParseError::invalid_bitstring("days");
        }
        switch (s[i]) {
        case '1':
            value |= 1 << i;
            break;
        case '0':
            break;
        default:
            throw ParseError::invalid_bitstring("days");
        }
    }

    return Days(value);
}

}