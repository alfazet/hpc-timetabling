#include <gtest/gtest.h>
#include <pugixml.hpp>

#include "parser/parse_error.hpp"
#include "parser/timeslots.hpp"

using namespace parser;

static pugi::xml_node load_node(pugi::xml_document &doc, const char *xml) {
    doc.load_string(xml);
    return doc.first_child();
}

TEST(TimeSlots, ValidTimeslot) {
    pugi::xml_document doc;
    auto node = load_node(
        doc,
        R"(<time start="90" length="10" days="1010100" weeks="1111111111111"/>)");

    auto ts = TimeSlots::parse(node);

    EXPECT_EQ(ts.start, 90u);
    EXPECT_EQ(ts.length, 10u);
    EXPECT_EQ(ts.days, Days::parse("1010100"));
    EXPECT_EQ(ts.weeks, Weeks::parse("1111111111111"));
}

TEST(TimeSlots, ValidTimeslotCustomName) {
    pugi::xml_document doc;
    auto node = load_node(
        doc,
        R"(<helloworld start="90" length="10" days="1010100" weeks="1111111111111"/>)");

    auto ts = TimeSlots::parse(node);

    EXPECT_EQ(ts.start, 90u);
    EXPECT_EQ(ts.length, 10u);
}

TEST(TimeSlots, FailsOnMissingStart) {
    pugi::xml_document doc;
    auto node = load_node(
        doc, R"(<time length="10" days="1010100" weeks="1111111111111"/>)");
    EXPECT_THROW(TimeSlots::parse(node), ParseError);
}

TEST(TimeSlots, FailsOnMissingLength) {
    pugi::xml_document doc;
    auto node = load_node(
        doc, R"(<time start="90" days="1010100" weeks="1111111111111"/>)");
    EXPECT_THROW(TimeSlots::parse(node), ParseError);
}

TEST(TimeSlots, FailsOnMissingDays) {
    pugi::xml_document doc;
    auto node =
        load_node(
            doc, R"(<time start="90" length="10" weeks="1111111111111"/>)");
    EXPECT_THROW(TimeSlots::parse(node), ParseError);
}

TEST(TimeSlots, FailsOnMissingWeeks) {
    pugi::xml_document doc;
    auto node =
        load_node(doc, R"(<time start="90" length="10" days="1010100"/>)");
    EXPECT_THROW(TimeSlots::parse(node), ParseError);
}

TEST(TimeSlots, FailsOnInvalidDays) {
    pugi::xml_document doc;
    auto node = load_node(
        doc,
        R"(<time start="90" length="10" days="abc" weeks="1111111111111"/>)");
    EXPECT_THROW(TimeSlots::parse(node), ParseError);
}

TEST(TimeSlots, FailsOnInvalidWeeks) {
    pugi::xml_document doc;
    auto node = load_node(
        doc, R"(<time start="90" length="10" days="1010100" weeks="xyz"/>)");
    EXPECT_THROW(TimeSlots::parse(node), ParseError);
}

TEST(TimeSlots, FailsOnInvalidStartNumber) {
    pugi::xml_document doc;
    auto node = load_node(
        doc,
        R"(<time start="abc" length="10" days="1010100" weeks="1111111111111"/>)");
    EXPECT_THROW(TimeSlots::parse(node), ParseError);
}
