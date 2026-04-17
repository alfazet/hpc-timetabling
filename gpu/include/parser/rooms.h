#ifndef GPU_TIMETABLING_ROOMS_H
#define GPU_TIMETABLING_ROOMS_H

#include <vector>

#include "typedefs.h"
#include "id_types.h"
#include "timeslots.h"

namespace pugi {
class xml_node;
}

namespace parser {
struct Travel {
    RoomId room;
    u32 value;

    bool operator==(const Travel &o) const {
        return room == o.room && value == o.value;
    }
};

struct Room {
    RoomId id;
    u32 capacity;
    std::vector<Travel> travels;
    std::vector<TimeSlots> unavailabilities;

    bool operator==(const Room &o) const {
        return id == o.id && capacity == o.capacity && travels == o.travels &&
               unavailabilities == o.unavailabilities;
    }
};

struct Rooms {
    std::vector<Room> items;

    static Rooms parse(const pugi::xml_node &node);

    bool operator==(const Rooms &o) const { return items == o.items; }
};

}

#endif //GPU_TIMETABLING_ROOMS_H