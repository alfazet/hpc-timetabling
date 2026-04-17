#include <cstring>
#include <pugixml.hpp>

#include "parser/utils.h"
#include "parser/parse_error.h"
#include "parser/optimization.h"

namespace parser {
Optimization Optimization::parse(const pugi::xml_node &node) {
    for (auto attr = node.first_attribute(); attr;
         attr = attr.next_attribute()) {
        const char *name = attr.name();
        if (std::strcmp(name, "time") != 0 && std::strcmp(name, "room") != 0 &&
            std::strcmp(name, "distribution") != 0 &&
            std::strcmp(name, "student") != 0) {
            throw ParseError::unexpected_attr(name);
        }
    }

    Optimization opt{};
    opt.time = utils::required_int<u32>(node, "time");
    opt.room = utils::required_int<u32>(node, "room");
    opt.distribution = utils::required_int<u32>(node, "distribution");
    opt.student = utils::required_int<u32>(node, "student");
    return opt;
}

}