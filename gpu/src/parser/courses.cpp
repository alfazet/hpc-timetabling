#include <cstring>
#include <pugixml.hpp>

#include "parser/utils.h"
#include "parser/courses.h"

namespace parser {
ClassRoom parse_class_room(const pugi::xml_node &node) {
    utils::reject_extra_attrs(node, {"id", "penalty"});
    ClassRoom cr;
    cr.room = RoomId::make(utils::required_int<u32>(node, "id"));
    cr.penalty = utils::required_int<u32>(node, "penalty");

    return cr;
}

ClassTime parse_class_time(const pugi::xml_node &node) {
    ClassTime ct;
    ct.penalty = utils::required_int<u32>(node, "penalty");
    ct.times = TimeSlots::parse(node);

    return ct;
}

Class parse_class(const pugi::xml_node &node) {
    for (auto attr = node.first_attribute(); attr;
         attr = attr.next_attribute()) {
        const char *name = attr.name();
        if (std::strcmp(name, "id") != 0 && std::strcmp(name, "limit") != 0 &&
            std::strcmp(name, "parent") != 0 && std::strcmp(name, "room") !=
            0) {
            throw ParseError::unexpected_attr(name);
        }
    }

    Class cls;
    cls.id = ClassId(utils::required_int<u32>(node, "id"));
    cls.limit = utils::optional_int<u32>(node, "limit");
    auto parent_attr = node.attribute("parent");
    if (parent_attr) {
        const char *val = parent_attr.value();
        char *end = nullptr;
        unsigned long v = std::strtoul(val, &end, 10);
        if (end == val || *end != '\0')
            throw ParseError::invalid_value("parent", val);
        cls.parent = ClassId(static_cast<u32>(v));
    }

    for (auto r : node.children("room")) {
        cls.rooms.push_back(parse_class_room(r));
    }
    for (auto t : node.children("time")) {
        cls.times.push_back(parse_class_time(t));
    }

    if (cls.times.empty()) {
        throw ParseError::missing_element("time");
    }

    return cls;
}

Subpart parse_subpart(const pugi::xml_node &node) {
    utils::reject_extra_attrs(node, {"id"});
    Subpart sp;
    sp.id = SubpartId::make(utils::required_int<u32>(node, "id"));
    for (auto c : node.children("class")) {
        sp.classes.push_back(parse_class(c));
    }

    return sp;
}

Config parse_config(const pugi::xml_node &node) {
    utils::reject_extra_attrs(node, {"id"});
    Config cfg;
    cfg.id = ConfigId::make(utils::required_int<u32>(node, "id"));
    for (auto s : node.children("subpart")) {
        cfg.subparts.push_back(parse_subpart(s));
    }

    return cfg;
}

Course parse_course(const pugi::xml_node &node) {
    utils::reject_extra_attrs(node, {"id"});
    Course course;
    course.id = CourseId(utils::required_int<u32>(node, "id"));
    for (auto c : node.children("config")) {
        course.configs.push_back(parse_config(c));
    }

    return course;
}

Courses Courses::parse(const pugi::xml_node &node) {
    if (node.first_attribute())
        throw ParseError::unexpected_attr("courses");
    Courses result;
    for (auto c : node.children("course")) {
        result.items.push_back(parse_course(c));
    }

    return result;
}

}