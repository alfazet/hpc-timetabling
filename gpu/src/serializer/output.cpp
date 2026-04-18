#include <sstream>

#include "parser/problem.h"
#include "serializer/utils.h"
#include "serializer/output.h"

namespace serializer {
OutputMetadata OutputMetadata::from_problem(const parser::Problem &problem) {
    return OutputMetadata{
        .name = problem.name,
        .runtime = 1.0f,
        .cores = 1,
        .technique = "Genetic Algorithm",
        .author = "todo",
        .institution = "todo",
        .country = "todo",
        .nr_days = problem.nr_days,
        .nr_weeks = problem.nr_weeks,
    };
}

std::string Output::serialize(const OutputMetadata &ctx) const {
    std::ostringstream os;
    os << R"(<?xml version="1.0" encoding="UTF-8"?>)" << '\n';
    os << R"(<!DOCTYPE solution PUBLIC)" << '\n'
        << R"(	        "-//ITC 2019//DTD Problem Format/EN")" << '\n'
        << R"(	        "http://www.itc2019.org/competition-format.dtd">)"
        << '\n';

    os << "<solution"
        << " name=\"" << ctx.name << "\""
        << " runtime=\"" << ctx.runtime << "\""
        << " cores=\"" << ctx.cores << "\""
        << " technique=\"" << ctx.technique << "\""
        << " author=\"" << ctx.author << "\""
        << " institution=\"" << ctx.institution << "\""
        << " country=\"" << ctx.country << "\">";

    for (const auto &[id, days, weeks, start, room, students] : classes) {
        os << "<class"
            << " id=\"" << id.value << "\""
            << " days=\"" << utils::bit_string<u8>(days.bits, ctx.nr_days)
            << "\""
            << " weeks=\"" << utils::bit_string<u16>(
                weeks.bits, ctx.nr_weeks)
            << "\""
            << " start=\"" << start << "\"";

        if (room.has_value())
            os << " room=\"" << room->value << "\"";

        if (students.empty()) {
            os << "/>";
        } else {
            os << ">";
            for (const auto &s : students)
                os << "<student id=\"" << s.id.value << "\"/>";
            os << "</class>";
        }
    }
    os << "</solution>";

    return os.str();
}

}
