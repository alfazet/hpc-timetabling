#ifndef GPU_TIMETABLING_OPTIMIZATION_H
#define GPU_TIMETABLING_OPTIMIZATION_H

#include "typedefs.h"

namespace pugi {
class xml_node;
}

namespace parser {

struct Optimization {
    u32 time;
    u32 room;
    u32 distribution;
    u32 student;

    static Optimization parse(const pugi::xml_node &node);

    bool operator==(const Optimization &o) const {
        return time == o.time && room == o.room && distribution == o.
               distribution &&
               student == o.student;
    }
};

}

#endif //GPU_TIMETABLING_OPTIMIZATION_H