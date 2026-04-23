#ifndef GPU_TIMETABLING_DAYS_H
#define GPU_TIMETABLING_DAYS_H

#include <string>

#include "typedefs.hpp"

namespace parser {
struct Days {
    u8 bits;

    explicit Days(u8 bits = 0) : bits(bits) {
    }

    static Days parse(const std::string &s);

    bool contains(u8 day) const { return (bits & (1 << day)) != 0; }

    bool operator==(const Days &o) const { return bits == o.bits; }
    bool operator!=(const Days &o) const { return !(*this == o); }
};

}

#endif //GPU_TIMETABLING_DAYS_H