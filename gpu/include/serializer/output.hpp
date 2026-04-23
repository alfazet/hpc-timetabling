#ifndef GPU_TIMETABLING_OUTPUT_H
#define GPU_TIMETABLING_OUTPUT_H

#include <optional>
#include <string>
#include <vector>

#include "parser/days.hpp"
#include "parser/id_types.hpp"
#include "parser/weeks.hpp"
#include "typedefs.hpp"

namespace parser {
struct Problem;
}

namespace serializer {
struct Student {
    parser::StudentId id;

    bool operator==(const Student &o) const { return id == o.id; }
};

struct Class {
    parser::ClassId id;
    parser::Days days;
    parser::Weeks weeks;
    u32 start;
    std::optional<parser::RoomId> room;
    std::vector<Student> students;

    bool operator==(const Class &o) const {
        return id == o.id && days == o.days && weeks == o.weeks &&
               start == o.start && room == o.room && students == o.students;
    }
};

struct OutputMetadata {
    std::string name;
    f32 runtime;
    usize cores;
    std::string technique;
    std::string author;
    std::string institution;
    std::string country;
    u32 nr_days;
    u32 nr_weeks;

    static OutputMetadata from_problem(const parser::Problem &problem);

    bool operator==(const OutputMetadata &o) const {
        return name == o.name && runtime == o.runtime && cores == o.cores &&
               technique == o.technique && author == o.author &&
               institution == o.institution && country == o.country &&
               nr_days == o.nr_days && nr_weeks == o.nr_weeks;
    }
};

struct Output {
    std::vector<Class> classes;

    std::string serialize(const OutputMetadata &ctx) const;

    bool operator==(const Output &o) const { return classes == o.classes; }
};

}

#endif // GPU_TIMETABLING_OUTPUT_H
