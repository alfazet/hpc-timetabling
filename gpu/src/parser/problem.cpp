#include <cstring>
#include <pugixml.hpp>

#include "parser/problem.hpp"

namespace parser {
template <typename T>
T require_attr(const pugi::xml_node &node, const char *name) {
    auto attr = node.attribute(name);
    if (!attr)
        throw ParseError::missing_attr(name);
    const char *val = attr.value();
    char *end = nullptr;
    unsigned long v = std::strtoul(val, &end, 10);
    if (end == val || *end != '\0')
        throw ParseError::invalid_value(name, val);

    return static_cast<T>(v);
}


Problem Problem::parse(const std::string &xml) {
    pugi::xml_document doc;
    auto parse_result = doc.load_string(xml.c_str());
    if (!parse_result)
        throw ParseError(std::string("XML parse error: ") +
                         parse_result.description());

    auto problem_node = doc.child("problem");
    if (!problem_node)
        throw ParseError::missing_element("problem");

    for (auto attr = problem_node.first_attribute(); attr;
         attr = attr.next_attribute()) {
        const char *name = attr.name();
        if (std::strcmp(name, "name") != 0 && std::strcmp(name, "nrDays") != 0
            &&
            std::strcmp(name, "nrWeeks") != 0 &&
            std::strcmp(name, "slotsPerDay") != 0) {
            throw ParseError::unexpected_attr(name);
        }
    }

    Problem p;
    auto name_attr = problem_node.attribute("name");
    if (!name_attr)
        throw ParseError::missing_attr("name");
    p.name = name_attr.value();
    p.nr_days = require_attr<u32>(problem_node, "nrDays");
    p.nr_weeks = require_attr<u32>(problem_node, "nrWeeks");
    p.slots_per_day = require_attr<u32>(problem_node, "slotsPerDay");

    auto opt_node = problem_node.child("optimization");
    if (!opt_node)
        throw ParseError::missing_element("optimization");
    p.optimization = Optimization::parse(opt_node);

    auto rooms_node = problem_node.child("rooms");
    if (!rooms_node)
        throw ParseError::missing_element("rooms");
    p.rooms = Rooms::parse(rooms_node);

    auto courses_node = problem_node.child("courses");
    if (!courses_node)
        throw ParseError::missing_element("courses");
    p.courses = Courses::parse(courses_node);

    auto dist_node = problem_node.child("distributions");
    if (!dist_node)
        throw ParseError::missing_element("distributions");
    p.distributions = Distributions::parse(dist_node);

    auto students_node = problem_node.child("students");
    if (students_node) {
        p.students = Students::parse(students_node);
    }

    return p;
}

}