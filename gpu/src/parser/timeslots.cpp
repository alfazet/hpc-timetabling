#include <string>
#include <pugixml.hpp>

#include "parser/utils.hpp"
#include "parser/parse_error.hpp"
#include "parser/timeslots.hpp"

namespace parser {
TimeSlots TimeSlots::parse(const pugi::xml_node &node) {
    TimeSlots ts;
    ts.start = utils::required_int<u32>(node, "start");
    ts.length = utils::required_int<u32>(node, "length");

    auto days_attr = node.attribute("days");
    if (!days_attr)
        throw ParseError::missing_attr("days");
    ts.days = Days::parse(days_attr.value());

    auto weeks_attr = node.attribute("weeks");
    if (!weeks_attr)
        throw ParseError::missing_attr("weeks");
    ts.weeks = Weeks::parse(weeks_attr.value());

    return ts;
}
}