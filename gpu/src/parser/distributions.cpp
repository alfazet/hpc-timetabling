#include <cstring>
#include <pugixml.hpp>

#include "parser/utils.hpp"
#include "parser/distributions.hpp"
#include "parser/parse_error.hpp"

namespace parser {
/// parse a single-parameter distribution
template <typename T>
T parse_single_param(const std::string &s, const char *prefix) {
    auto plen = std::strlen(prefix);
    if (s.size() <= plen + 2 || s[plen] != '(' || s.back() != ')')
        throw ParseError::invalid_value(prefix, s);

    auto inner = s.substr(plen + 1, s.size() - plen - 2);
    char *end = nullptr;
    unsigned long v = std::strtoul(inner.c_str(), &end, 10);
    if (end == inner.c_str() || *end != '\0')
        throw ParseError::invalid_value(prefix, s);

    return static_cast<T>(v);
}

/// parse a two-parameter distribution
std::pair<u16, u16> parse_two_params(const std::string &s,
                                     const char *prefix) {
    auto pref_len = std::strlen(prefix);
    if (s.size() <= pref_len + 2 || s[pref_len] != '(' || s.back() != ')')
        throw ParseError::invalid_value(prefix, s);

    auto inner = s.substr(pref_len + 1, s.size() - pref_len - 2);
    auto comma = inner.find(',');
    if (comma == std::string::npos)
        throw ParseError::invalid_value(prefix, s);
    auto first_s = inner.substr(0, comma);
    auto second_s = inner.substr(comma + 1);
    if (second_s.find(',') != std::string::npos)
        throw ParseError::invalid_value(prefix, s);

    char *end = nullptr;
    unsigned long a = std::strtoul(first_s.c_str(), &end, 10);
    if (end == first_s.c_str() || *end != '\0')
        throw ParseError::invalid_value(prefix, s);

    unsigned long b = std::strtoul(second_s.c_str(), &end, 10);
    if (end == second_s.c_str() || *end != '\0')
        throw ParseError::invalid_value(prefix, s);

    return {static_cast<u16>(a), static_cast<u16>(b)};
}


DistributionKind parse_distribution_kind(const std::string &s) {
    if (s == "SameStart")
        return SameStart{};
    if (s == "SameTime")
        return SameTime{};
    if (s == "DifferentTime")
        return DifferentTime{};
    if (s == "SameDays")
        return SameDays{};
    if (s == "DifferentDays")
        return DifferentDays{};
    if (s == "SameWeeks")
        return SameWeeks{};
    if (s == "DifferentWeeks")
        return DifferentWeeks{};
    if (s == "Overlap")
        return Overlap{};
    if (s == "NotOverlap")
        return NotOverlap{};
    if (s == "SameRoom")
        return SameRoom{};
    if (s == "DifferentRoom")
        return DifferentRoom{};
    if (s == "SameAttendees")
        return SameAttendees{};
    if (s == "Precedence")
        return Precedence{};

    if (s.rfind("WorkDay", 0) == 0)
        return WorkDay{parse_single_param<u16>(s, "WorkDay")};
    if (s.rfind("MinGap", 0) == 0)
        return MinGap{parse_single_param<u16>(s, "MinGap")};
    if (s.rfind("MaxDays", 0) == 0)
        return MaxDays{parse_single_param<u8>(s, "MaxDays")};
    if (s.rfind("MaxDayLoad", 0) == 0)
        return MaxDayLoad{parse_single_param<u16>(s, "MaxDayLoad")};
    if (s.rfind("MaxBreaks", 0) == 0) {
        auto [r, sv] = parse_two_params(s, "MaxBreaks");
        return MaxBreaks{r, sv};
    }
    if (s.rfind("MaxBlock", 0) == 0) {
        auto [m, sv] = parse_two_params(s, "MaxBlock");
        return MaxBlock{m, sv};
    }

    throw ParseError::invalid_value("type", s);
}

Distributions Distributions::parse(const pugi::xml_node &node) {
    if (node.first_attribute())
        throw ParseError::unexpected_attr("distributions");

    Distributions result;
    for (auto dist_node : node.children("distribution")) {
        Distribution dist;
        auto type_attr = dist_node.attribute("type");
        if (!type_attr)
            throw ParseError::missing_attr("type");
        dist.kind = parse_distribution_kind(type_attr.value());

        auto penalty_attr = dist_node.attribute("penalty");
        if (penalty_attr) {
            const char *val = penalty_attr.value();
            char *end = nullptr;
            unsigned long v = std::strtoul(val, &end, 10);
            if (end == val || *end != '\0')
                throw ParseError::invalid_value("penalty", val);
            dist.penalty = static_cast<u32>(v);
        }

        for (auto attr = dist_node.first_attribute(); attr;
             attr = attr.next_attribute()) {
            const char *name = attr.name();
            if (std::strcmp(name, "type") != 0 && std::strcmp(name, "penalty")
                != 0 &&
                std::strcmp(name, "required") != 0) {
                throw ParseError::unexpected_attr(name);
            }
        }

        for (auto cls : dist_node.children("class")) {
            for (auto attr = cls.first_attribute(); attr;
                 attr = attr.next_attribute()) {
                if (std::strcmp(attr.name(), "id") != 0)
                    throw ParseError::unexpected_attr(attr.name());
            }
            dist.classes.
                push_back(ClassId::make(utils::required_int<u32>(cls, "id")));
        }
        result.items.push_back(std::move(dist));
    }

    return result;
}

}