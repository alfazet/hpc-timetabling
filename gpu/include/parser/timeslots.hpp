#ifndef GPU_TIMETABLING_TIMESLOTS_H
#define GPU_TIMETABLING_TIMESLOTS_H

#include "typedefs.hpp"
#include "days.hpp"
#include "weeks.hpp"

namespace pugi {
class xml_node;
}

namespace parser {
struct TimeSlots {
    u32 start;
    u32 length;
    Days days;
    Weeks weeks;

    static TimeSlots parse(const pugi::xml_node &node);

    bool operator==(const TimeSlots &o) const {
        return start == o.start && length == o.length && days == o.days &&
               weeks == o.weeks;
    }

    bool operator!=(const TimeSlots &o) const { return !(*this == o); }
};

}

#endif //GPU_TIMETABLING_TIMESLOTS_H