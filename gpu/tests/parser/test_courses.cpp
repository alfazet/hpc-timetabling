#include <gtest/gtest.h>
#include <pugixml.hpp>

#include "parser/parse_error.h"
#include "parser/courses.h"

using namespace parser;

static pugi::xml_node load_node(pugi::xml_document &doc, const char *xml) {
    doc.load_string(xml);
    return doc.first_child();
}

TEST(Courses, SingleCourseStructure) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(
        <courses>
            <course id="1">
                <config id="1">
                    <subpart id="1">
                        <class id="1" limit="20">
                            <time days="1000000" start="10" length="5" weeks="1111111111111" penalty="0"/>
                        </class>
                    </subpart>
                </config>
            </course>
        </courses>
    )");

    auto courses = Courses::parse(node);

    ASSERT_EQ(courses.items.size(), 1u);
    auto &course = courses.items[0];
    EXPECT_EQ(course.id, CourseId(1));
    ASSERT_EQ(course.configs.size(), 1u);

    auto &config = course.configs[0];
    EXPECT_EQ(config.id, ConfigId(1));
    ASSERT_EQ(config.subparts.size(), 1u);

    auto &subpart = config.subparts[0];
    EXPECT_EQ(subpart.id, SubpartId(1));
    ASSERT_EQ(subpart.classes.size(), 1u);

    auto &cls = subpart.classes[0];
    EXPECT_EQ(cls.id, ClassId(1));
    EXPECT_EQ(cls.limit, std::optional<uint32_t>(20));
    EXPECT_EQ(cls.times.size(), 1u);
}

TEST(Courses, ClassWithRoom) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(
        <courses>
            <course id="1">
                <config id="1">
                    <subpart id="1">
                        <class id="1" limit="10">
                            <room id="5" penalty="2"/>
                            <time days="1000000" start="10" length="5" weeks="1111111111111" penalty="0"/>
                        </class>
                    </subpart>
                </config>
            </course>
        </courses>
    )");

    auto courses = Courses::parse(node);
    auto &cls = courses.items[0].configs[0].subparts[0].classes[0];

    ASSERT_EQ(cls.rooms.size(), 1u);
    EXPECT_EQ(cls.rooms[0].room, RoomId::make(5));
    EXPECT_EQ(cls.rooms[0].penalty, 2u);
}

TEST(Courses, ClassWithParent) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(
        <courses>
            <course id="1">
                <config id="1">
                    <subpart id="1">
                        <class id="1" limit="20">
                            <time days="1000000" start="10" length="5" weeks="1111111111111" penalty="0"/>
                        </class>
                        <class id="2" limit="20" parent="1">
                            <time days="0100000" start="20" length="5" weeks="1111111111111" penalty="0"/>
                        </class>
                    </subpart>
                </config>
            </course>
        </courses>
    )");

    auto courses = Courses::parse(node);
    auto &classes = courses.items[0].configs[0].subparts[0].classes;

    EXPECT_EQ(classes[1].parent, std::optional<ClassId>(ClassId(1)));
}

TEST(Courses, ClassMustHaveTime) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(
        <courses>
            <course id="1">
                <config id="1">
                    <subpart id="1">
                        <class id="1" limit="20">
                        </class>
                    </subpart>
                </config>
            </course>
        </courses>
    )");

    EXPECT_THROW(Courses::parse(node), ParseError);
}

TEST(Courses, MultipleCourses) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(
        <courses>
            <course id="1">
                <config id="1">
                    <subpart id="1">
                        <class id="1" limit="10">
                            <time days="1000000" start="10" length="5" weeks="1111111111111" penalty="0"/>
                        </class>
                    </subpart>
                </config>
            </course>
            <course id="2">
                <config id="2">
                    <subpart id="3">
                        <class id="5" limit="15">
                            <time days="0100000" start="20" length="6" weeks="1111111111111" penalty="1"/>
                        </class>
                    </subpart>
                </config>
            </course>
        </courses>
    )");

    auto courses = Courses::parse(node);
    ASSERT_EQ(courses.items.size(), 2u);
    EXPECT_EQ(courses.items[0].id, CourseId(1));
    EXPECT_EQ(courses.items[1].id, CourseId(2));
}