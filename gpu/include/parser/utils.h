#ifndef GPU_TIMETABLING_UTILS_H
#define GPU_TIMETABLING_UTILS_H

#include <optional>

namespace pugi {
class xml_node;
}

namespace parser::utils {
template <typename T>
T required_int(const pugi::xml_node &node, const char *name);

template <typename T>
std::optional<T> optional_int(const pugi::xml_node &node,
                              const char *name);

void reject_extra_attrs(const pugi::xml_node &node,
                        std::initializer_list<const char *> allowed);

std::string read_file(const std::string &path);
}

#endif //GPU_TIMETABLING_UTILS_H