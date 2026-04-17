#ifndef GPU_TIMETABLING_WEEKS_H
#define GPU_TIMETABLING_WEEKS_H

#include <string>

#include "typedefs.h"
#include "parse_error.h"

namespace parser {
struct Weeks {
    u16 bits;

    explicit Weeks(u16 bits = 0) : bits(bits) {
    }

    static Weeks parse(const std::string &s);

    bool contains(uint8_t week) const { return (bits & (1 << week)) != 0; }

    bool operator==(const Weeks &o) const { return bits == o.bits; }
    bool operator!=(const Weeks &o) const { return !(*this == o); }
};

}

#endif //GPU_TIMETABLING_WEEKS_H