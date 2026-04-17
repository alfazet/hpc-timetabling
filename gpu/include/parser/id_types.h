#ifndef GPU_TIMETABLING_ID_TYPES_H
#define GPU_TIMETABLING_ID_TYPES_H

#include <cassert>

#include "typedefs.h"

namespace parser {
struct RoomId {
    usize value;

    explicit RoomId(usize v = 0) : value(v) {
    }

    static RoomId make(usize v) {
        assert(v >= 1);
        return RoomId(v);
    }

    bool operator==(const RoomId &o) const { return value == o.value; }
    bool operator!=(const RoomId &o) const { return value != o.value; }
};

struct CourseId {
    usize value;

    explicit CourseId(usize v = 0) : value(v) {
    }

    static CourseId make(usize v) {
        assert(v >= 1);
        return CourseId(v);
    }

    bool operator==(const CourseId &o) const { return value == o.value; }
    bool operator!=(const CourseId &o) const { return value != o.value; }
};

struct ConfigId {
    usize value;

    explicit ConfigId(usize v = 0) : value(v) {
    }

    static ConfigId make(usize v) {
        assert(v >= 1);
        return ConfigId(v);
    }

    bool operator==(const ConfigId &o) const { return value == o.value; }
    bool operator!=(const ConfigId &o) const { return value != o.value; }
};

struct SubpartId {
    usize value;

    explicit SubpartId(usize v = 0) : value(v) {
    }

    static SubpartId make(usize v) {
        assert(v >= 1);
        return SubpartId(v);
    }

    bool operator==(const SubpartId &o) const { return value == o.value; }
    bool operator!=(const SubpartId &o) const { return value != o.value; }
};

struct ClassId {
    usize value;

    explicit ClassId(usize v = 0) : value(v) {
    }

    static ClassId make(usize v) {
        assert(v >= 1);
        return ClassId(v);
    }

    bool operator==(const ClassId &o) const { return value == o.value; }
    bool operator!=(const ClassId &o) const { return value != o.value; }
};

struct StudentId {
    usize value;

    explicit StudentId(usize v = 0) : value(v) {
    }

    static StudentId make(usize v) {
        assert(v >= 1);
        return StudentId(v);
    }

    bool operator==(const StudentId &o) const { return value == o.value; }
    bool operator!=(const StudentId &o) const { return value != o.value; }
};

}

#endif //GPU_TIMETABLING_ID_TYPES_H