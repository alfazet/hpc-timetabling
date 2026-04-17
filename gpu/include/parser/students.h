#ifndef GPU_TIMETABLING_STUDENTS_H
#define GPU_TIMETABLING_STUDENTS_H

#include <vector>

#include "id_types.h"

namespace pugi {
class xml_node;
}

namespace parser {
struct Student {
    StudentId id;
    std::vector<CourseId> courses;

    bool operator==(const Student &o) const {
        return id == o.id && courses == o.courses;
    }
};

struct Students {
    std::vector<Student> items;

    static Students parse(const pugi::xml_node &node);

    bool operator==(const Students &o) const { return items == o.items; }
};

}

#endif //GPU_TIMETABLING_STUDENTS_H