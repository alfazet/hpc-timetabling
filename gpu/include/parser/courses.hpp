#ifndef GPU_TIMETABLING_COURSES_H
#define GPU_TIMETABLING_COURSES_H

#include <optional>
#include <vector>

#include "typedefs.hpp"
#include "id_types.hpp"
#include "timeslots.hpp"

namespace pugi {
class xml_node;
}

namespace parser {
struct ClassRoom {
    RoomId room;
    u32 penalty;

    bool operator==(const ClassRoom &o) const {
        return room == o.room && penalty == o.penalty;
    }
};

struct ClassTime {
    TimeSlots times;
    u32 penalty;

    bool operator==(const ClassTime &o) const {
        return times == o.times && penalty == o.penalty;
    }
};

struct Class {
    ClassId id;
    std::optional<u32> limit;
    std::optional<ClassId> parent;
    std::vector<ClassRoom> rooms;
    std::vector<ClassTime> times;

    bool operator==(const Class &o) const {
        return id == o.id && limit == o.limit && parent == o.parent &&
               rooms == o.rooms && times == o.times;
    }
};

struct Subpart {
    SubpartId id;
    std::vector<Class> classes;

    bool operator==(const Subpart &o) const {
        return id == o.id && classes == o.classes;
    }
};

struct Config {
    ConfigId id;
    std::vector<Subpart> subparts;

    bool operator==(const Config &o) const {
        return id == o.id && subparts == o.subparts;
    }
};

struct Course {
    CourseId id;
    std::vector<Config> configs;

    bool operator==(const Course &o) const {
        return id == o.id && configs == o.configs;
    }
};

struct Courses {
    std::vector<Course> items;

    static Courses parse(const pugi::xml_node &node);

    bool operator==(const Courses &o) const { return items == o.items; }
};

}

#endif //GPU_TIMETABLING_COURSES_H