#include <gtest/gtest.h>
#include <pugixml.hpp>

#include "parser/parse_error.hpp"
#include "parser/students.hpp"

using namespace parser;

static pugi::xml_node load_node(pugi::xml_document &doc, const char *xml) {
    doc.load_string(xml);
    return doc.first_child();
}

TEST(Students, ParseStudents) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(
        <students>
            <student id="1">
                <course id="1"/>
                <course id="5"/>
            </student>
            <student id="2">
                <course id="1"/>
                <course id="3"/>
                <course id="4"/>
            </student>
        </students>
    )");

    auto students = Students::parse(node);
    ASSERT_EQ(students.items.size(), 2u);

    EXPECT_EQ(students.items[0].id, StudentId(1));
    ASSERT_EQ(students.items[0].courses.size(), 2u);
    EXPECT_EQ(students.items[0].courses[0], CourseId::make(1));
    EXPECT_EQ(students.items[0].courses[1], CourseId::make(5));

    EXPECT_EQ(students.items[1].id, StudentId(2));
    ASSERT_EQ(students.items[1].courses.size(), 3u);
    EXPECT_EQ(students.items[1].courses[0], CourseId::make(1));
    EXPECT_EQ(students.items[1].courses[1], CourseId::make(3));
    EXPECT_EQ(students.items[1].courses[2], CourseId::make(4));
}

TEST(Students, EmptyStudents) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(<students></students>)");
    auto students = Students::parse(node);
    EXPECT_TRUE(students.items.empty());
}

TEST(Students, StudentWithNoCourses) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(
        <students>
            <student id="1"></student>
        </students>
    )");

    auto students = Students::parse(node);
    ASSERT_EQ(students.items.size(), 1u);
    EXPECT_TRUE(students.items[0].courses.empty());
}

TEST(Students, MissingIdFails) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(
        <students>
            <student>
                <course id="1"/>
            </student>
        </students>
    )");
    EXPECT_THROW(Students::parse(node), ParseError);
}

TEST(Students, CourseMissingIdFails) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(
        <students>
            <student id="1">
                <course/>
            </student>
        </students>
    )");
    EXPECT_THROW(Students::parse(node), ParseError);
}
