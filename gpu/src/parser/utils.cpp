#include <cstring>
#include <fstream>
#include <sstream>
#include <optional>
#include <pugixml.hpp>

#include "typedefs.h"
#include "parser/parse_error.h"
#include "parser/utils.h"

namespace parser::utils {
template <typename T>
T required_int(const pugi::xml_node &node, const char *name) {
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

template <typename T>
std::optional<T> optional_int(const pugi::xml_node &node,
                              const char *name) {
    auto attr = node.attribute(name);
    if (!attr)
        return std::nullopt;
    const char *val = attr.value();
    char *end = nullptr;
    unsigned long v = std::strtoul(val, &end, 10);
    if (end == val || *end != '\0')
        throw ParseError::invalid_value(name, val);

    return static_cast<T>(v);
}

void reject_extra_attrs(const pugi::xml_node &node,
                        std::initializer_list<const char *> allowed) {
    for (auto attr = node.first_attribute(); attr;
         attr = attr.next_attribute()) {
        bool ok = false;
        for (auto *a : allowed)
            if (std::strcmp(attr.name(), a) == 0) {
                ok = true;
                break;
            }
        if (!ok)
            throw ParseError::unexpected_attr(attr.name());
    }
}

std::string read_file(const std::string &path) {
    std::ifstream file(path);
    if (!file.is_open())
        throw ParseError("cannot open file " + path);
    std::ostringstream ss;
    ss << file.rdbuf();

    return ss.str();
}

// Either this or all templated functions would have to
// be specified in the header file.
// In other words: great language design.
template u32 required_int<u32>(const pugi::xml_node &, const char *);

template std::optional<u32> optional_int<u32>(const pugi::xml_node &,
                                              const char *);

}