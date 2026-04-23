#include <gtest/gtest.h>

#include "executor/model.hpp"
#include "parser/parser.hpp"

static TimetableData sample_data() {
    auto content = parser::utils::read_file(
        std::string(DATA_DIR) + "/itc2019/sample.xml");
    auto problem = parser::Problem::parse(content);

    return TimetableData::from_problem(problem);
}

TEST(Model, CourseCount) {
    auto data = sample_data();
    ASSERT_EQ(data.courses.size(), 1u);
}

TEST(Model, ConfigCount) {
    auto data = sample_data();
    const auto &course = data.courses.at(0);
    ASSERT_EQ(course.configs_end - course.configs_start, 1u);
}

TEST(Model, SubpartCount) {
    auto data = sample_data();
    const auto &config = data.configs.at(0);
    ASSERT_EQ(config.subparts_end - config.subparts_start, 2u);
}

TEST(Model, HasParent) {
    auto data = sample_data();
    auto parent_idx = data.classes.at(3 - 1).parent.value();
    ASSERT_EQ(data.classes.at(parent_idx).id, parser::ClassId::make(1));
}

TEST(Model, HasNoParent) {
    auto data = sample_data();
    ASSERT_FALSE(data.classes.at(2 - 1).parent.has_value());
}