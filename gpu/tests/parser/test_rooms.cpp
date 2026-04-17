#include <gtest/gtest.h>
#include <pugixml.hpp>

#include "parser/parse_error.h"
#include "parser/rooms.h"

using namespace parser;

static pugi::xml_node load_node(pugi::xml_document &doc, const char *xml) {
    doc.load_string(xml);
    return doc.first_child();
}

TEST(Rooms, EmptyRooms) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(<rooms></rooms>)");
    auto rooms = Rooms::parse(node);
    EXPECT_TRUE(rooms.items.empty());
}

TEST(Rooms, MultipleRooms) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(
        <rooms>
            <room id="1" capacity="100"/>
            <room id="2" capacity="200"></room>
            <room id="3" capacity="300">
                <unavailable days="1100000" start="102" length="24" weeks="1000000000000"/>
            </room>
        </rooms>
    )");

    auto rooms = Rooms::parse(node);

    ASSERT_EQ(rooms.items.size(), 3u);
    EXPECT_EQ(rooms.items[0].id, RoomId(1));
    EXPECT_EQ(rooms.items[0].capacity, 100u);
    EXPECT_EQ(rooms.items[1].id, RoomId(2));
    EXPECT_EQ(rooms.items[1].capacity, 200u);
    EXPECT_EQ(rooms.items[2].id, RoomId(3));
    EXPECT_EQ(rooms.items[2].capacity, 300u);

    ASSERT_EQ(rooms.items[2].unavailabilities.size(), 1u);
    auto &u = rooms.items[2].unavailabilities[0];
    EXPECT_EQ(u.start, 102u);
    EXPECT_EQ(u.length, 24u);
    EXPECT_EQ(u.days, Days(1 << 0 | 1 << 1));
    EXPECT_EQ(u.weeks, Weeks(1));
}

TEST(Rooms, UnexpectedAttribute) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(<rooms foo="bar"></rooms>)");
    EXPECT_THROW(Rooms::parse(node), ParseError);
}