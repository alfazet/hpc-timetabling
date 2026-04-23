#include <gtest/gtest.h>
#include <pugixml.hpp>

#include "parser/optimization.hpp"
#include "parser/parse_error.hpp"

using namespace parser;

static pugi::xml_node load_node(pugi::xml_document &doc, const char *xml) {
    doc.load_string(xml);
    return doc.first_child();
}

TEST(Optimization, ValidOptimization) {
    pugi::xml_document doc;
    auto node = load_node(
        doc,
        R"(<optimization time="1" room="2" distribution="3" student="4"/>)");

    auto opt = Optimization::parse(node);

    EXPECT_EQ(opt.time, 1u);
    EXPECT_EQ(opt.room, 2u);
    EXPECT_EQ(opt.distribution, 3u);
    EXPECT_EQ(opt.student, 4u);
}

TEST(Optimization, MissingTimeAttr) {
    pugi::xml_document doc;
    auto node = load_node(
        doc, R"(<optimization room="2" distribution="3" student="4"/>)");
    EXPECT_THROW(Optimization::parse(node), ParseError);
}

TEST(Optimization, UnexpectedAttribute) {
    pugi::xml_document doc;
    auto node = load_node(
        doc,
        R"(<optimization time="1" room="2" distribution="3" student="4" foo="5"/>)");
    EXPECT_THROW(Optimization::parse(node), ParseError);
}

TEST(Optimization, InvalidIntegerValue) {
    pugi::xml_document doc;
    auto node = load_node(
        doc,
        R"(<optimization time="x" room="2" distribution="3" student="4"/>)");
    EXPECT_THROW(Optimization::parse(node), ParseError);
}
