#include "parser/weeks.h"

namespace parser {
Weeks Weeks::parse(const std::string &s) {
    u16 value = 0;
    for (usize i = 0; i < s.size(); i++) {
        switch (s[i]) {
        case '1':
            value |= 1 << i;
            break;
        case '0':
            break;
        default:
            throw ParseError::invalid_bitstring("weeks");
        }
    }

    return Weeks(value);
}
}