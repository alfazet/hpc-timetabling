#ifndef GPU_TIMETABLING_DISTRIBUTIONS_H
#define GPU_TIMETABLING_DISTRIBUTIONS_H

#include <optional>
#include <string>
#include <variant>
#include <vector>

#include "typedefs.hpp"
#include "id_types.hpp"

namespace pugi {
class xml_node;
}

namespace parser {
struct SameStart {
};

struct SameTime {
};

struct DifferentTime {
};

struct SameDays {
};

struct DifferentDays {
};

struct SameWeeks {
};

struct DifferentWeeks {
};

struct Overlap {
};

struct NotOverlap {
};

struct SameRoom {
};

struct DifferentRoom {
};

struct SameAttendees {
};

struct Precedence {
};

struct WorkDay {
    u16 s;
};

struct MinGap {
    u16 g;
};

struct MaxDays {
    u8 d;
};

struct MaxDayLoad {
    u16 s;
};

struct MaxBreaks {
    u16 r;
    u16 s;
};

struct MaxBlock {
    u16 m;
    u16 s;
};

using DistributionKind =
std::variant<SameStart, SameTime, DifferentTime, SameDays, DifferentDays,
             SameWeeks, DifferentWeeks, Overlap, NotOverlap, SameRoom,
             DifferentRoom, SameAttendees, Precedence, WorkDay, MinGap,
             MaxDays, MaxDayLoad, MaxBreaks, MaxBlock>;

DistributionKind parse_distribution_kind(const std::string &s);

struct Distribution {
    DistributionKind kind;
    std::vector<ClassId> classes;
    std::optional<u32> penalty;

    bool required() const { return !penalty.has_value(); }
};

struct Distributions {
    std::vector<Distribution> items;

    static Distributions parse(const pugi::xml_node &node);
};

}

#endif //GPU_TIMETABLING_DISTRIBUTIONS_H