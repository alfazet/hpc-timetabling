#include <gtest/gtest.h>
#include <filesystem>

#include "parser/parser.hpp"
#include "parser/utils.hpp"

using namespace parser;

TEST(Problem, SampleXml) {
    std::string path = std::string(DATA_DIR) + "/itc2019/sample.xml";
    std::string xml = utils::read_file(path);
    auto problem = Problem::parse(xml);

    EXPECT_EQ(problem.name, "unique-instance-name");
    EXPECT_EQ(problem.nr_days, 7u);
    EXPECT_EQ(problem.nr_weeks, 13u);
    EXPECT_EQ(problem.slots_per_day, 288u);

    EXPECT_EQ(problem.optimization.time, 2u);
    EXPECT_EQ(problem.optimization.room, 1u);
    EXPECT_EQ(problem.optimization.distribution, 1u);
    EXPECT_EQ(problem.optimization.student, 2u);

    ASSERT_EQ(problem.rooms.items.size(), 3u);
    EXPECT_EQ(problem.rooms.items[0].id, RoomId(1));
    EXPECT_EQ(problem.rooms.items[0].capacity, 50u);
    EXPECT_EQ(problem.rooms.items[1].id, RoomId(2));
    EXPECT_EQ(problem.rooms.items[1].capacity, 100u);
    ASSERT_EQ(problem.rooms.items[1].travels.size(), 1u);
    EXPECT_EQ(problem.rooms.items[1].travels[0].room, RoomId(1));
    EXPECT_EQ(problem.rooms.items[1].travels[0].value, 2u);
    EXPECT_EQ(problem.rooms.items[2].id, RoomId(3));
    EXPECT_EQ(problem.rooms.items[2].capacity, 80u);
    ASSERT_EQ(problem.rooms.items[2].unavail.size(), 2u);

    ASSERT_EQ(problem.courses.items.size(), 1u);
    auto &course = problem.courses.items[0];
    EXPECT_EQ(course.id, CourseId(1));
    ASSERT_EQ(course.configs.size(), 1u);
    ASSERT_EQ(course.configs[0].subparts.size(), 2u);

    auto &sp1 = course.configs[0].subparts[0];
    ASSERT_EQ(sp1.classes.size(), 2u);
    EXPECT_EQ(sp1.classes[0].id, ClassId(1));
    EXPECT_EQ(sp1.classes[0].limit, std::optional<uint32_t>(20));
    ASSERT_EQ(sp1.classes[0].rooms.size(), 2u);
    ASSERT_EQ(sp1.classes[0].times.size(), 2u);

    auto &sp2 = course.configs[0].subparts[1];
    ASSERT_EQ(sp2.classes.size(), 1u);
    EXPECT_EQ(sp2.classes[0].id, ClassId(3));
    EXPECT_FALSE(sp2.classes[0].limit.has_value());
    EXPECT_EQ(sp2.classes[0].parent, std::optional(ClassId(1)));

    ASSERT_EQ(problem.distributions.items.size(), 4u);
    EXPECT_TRUE(
        std::holds_alternative<NotOverlap>(problem.distributions.items[0].kind
        ));
    EXPECT_FALSE(problem.distributions.items[0].penalty.has_value());

    EXPECT_TRUE(
        std::holds_alternative<Precedence>(problem.distributions.items[1].kind
        ));
    EXPECT_EQ(problem.distributions.items[1].penalty,
              std::optional<u32>(2));

    EXPECT_TRUE(std::holds_alternative<SameAttendees>(
        problem.distributions.items[2].kind));

    auto &md = std::get<MaxDays>(problem.distributions.items[3].kind);
    EXPECT_EQ(md.d, 2);

    ASSERT_EQ(problem.students.items.size(), 2u);
    EXPECT_EQ(problem.students.items[0].id, StudentId(1));
    ASSERT_EQ(problem.students.items[0].courses.size(), 2u);
    EXPECT_EQ(problem.students.items[1].id, StudentId(2));
    ASSERT_EQ(problem.students.items[1].courses.size(), 3u);
}