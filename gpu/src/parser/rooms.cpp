#include <pugixml.hpp>

#include "parser/utils.hpp"
#include "parser/parse_error.hpp"
#include "parser/rooms.hpp"

namespace parser {
Rooms Rooms::parse(const pugi::xml_node &node) {
    if (node.first_attribute())
        throw ParseError::unexpected_attr("rooms");

    Rooms result;
    for (auto child : node.children("room")) {
        Room room;
        utils::reject_extra_attrs(child, {"id", "capacity"});

        room.id = RoomId(utils::required_int<u32>(child, "id"));
        room.capacity = utils::required_int<u32>(child, "capacity");
        for (auto t : child.children("travel")) {
            utils::reject_extra_attrs(t, {"room", "value"});
            Travel tr;
            tr.room = RoomId(utils::required_int<u32>(t, "room"));
            tr.value = utils::required_int<u32>(t, "value");
            room.travels.push_back(tr);
        }

        for (auto u : child.children("unavailable")) {
            room.unavailabilities.push_back(TimeSlots::parse(u));
        }
        result.items.push_back(std::move(room));
    }

    return result;
}

}