#include <gtest/gtest.h>

#include "serializer/serializer.hpp"

using namespace serializer;

TEST(SerializerUtils, BitStringU8_Basic) {
    u8 bits = (1u << 0) | (1u << 1);
    EXPECT_EQ(utils::bit_string<u8>(bits, 7), "1100000");
}

TEST(SerializerUtils, BitStringU8_AllZero) {
    EXPECT_EQ(utils::bit_string<u8>(0, 7), "0000000");
}

TEST(SerializerUtils, BitStringU8_AllOne) {
    EXPECT_EQ(utils::bit_string<u8>(0x7f, 7), "1111111");
}

TEST(SerializerUtils, BitStringU8_ShorterLen) {
    EXPECT_EQ(utils::bit_string<u8>(0b1111, 3), "111");
}

TEST(SerializerUtils, BitStringU16_Basic) {
    u16 all13 = (1u << 13) - 1;
    EXPECT_EQ(utils::bit_string<u16>(all13, 13), "1111111111111");
}

TEST(SerializerUtils, BitStringU16_Alternating) {
    EXPECT_EQ(utils::bit_string<u16>(0b1010101010, 10), "0101010101");
}

TEST(OutputSerialize, EmptyOutput) {
    Output out;
    OutputMetadata ctx{
        .name = "test-instance",
        .runtime = 2.5f,
        .cores = 4,
        .technique = "GA",
        .author = "foo",
        .institution = "bar",
        .country = "baz",
        .nr_days = 7,
        .nr_weeks = 13,
    };

    std::string s = out.serialize(ctx);

    EXPECT_NE(s.find("<?xml"), std::string::npos);
    EXPECT_NE(s.find("<!DOCTYPE solution"), std::string::npos);
    EXPECT_NE(s.find(R"(name="test-instance")"), std::string::npos);
    EXPECT_NE(s.find(R"(runtime="2.5")"), std::string::npos);
    EXPECT_NE(s.find(R"(cores="4")"), std::string::npos);
    EXPECT_NE(s.find(R"(technique="GA")"), std::string::npos);
    EXPECT_NE(s.find("</solution>"), std::string::npos);
}

TEST(OutputSerialize, ClassWithoutRoom) {
    Output out;
    out.classes.push_back(Class{
        .id = parser::ClassId(1),
        .days = parser::Days((1u << 0) | (1u << 2)), // days 0 and 2
        .weeks = parser::Weeks(0x1FFF), // all 13 weeks
        .start = 90,
        .room = std::nullopt,
        .students = {},
    });

    OutputMetadata ctx{.name = "i",
                       .runtime = 1.0f,
                       .cores = 1,
                       .technique = "t",
                       .author = "a",
                       .institution = "i",
                       .country = "c",
                       .nr_days = 7,
                       .nr_weeks = 13};
    std::string s = out.serialize(ctx);

    EXPECT_NE(s.find(R"(id="1")"), std::string::npos);
    EXPECT_NE(s.find(R"(days="1010000")"), std::string::npos);
    EXPECT_NE(s.find(R"(weeks="1111111111111")"), std::string::npos);
    EXPECT_NE(s.find(R"(start="90")"), std::string::npos);
    EXPECT_EQ(s.find(R"(room=")"), std::string::npos);
}

TEST(OutputSerialize, ClassWithRoom) {
    Output out;
    out.classes.push_back(Class{
        .id = parser::ClassId(3),
        .days = parser::Days(1u),
        .weeks = parser::Weeks(1u),
        .start = 50,
        .room = parser::RoomId(7),
        .students = {},
    });

    OutputMetadata ctx{.name = "i",
                       .runtime = 1.0f,
                       .cores = 1,
                       .technique = "t",
                       .author = "a",
                       .institution = "i",
                       .country = "c",
                       .nr_days = 7,
                       .nr_weeks = 13};
    std::string s = out.serialize(ctx);

    EXPECT_NE(s.find(R"(room="7")"), std::string::npos);
}

TEST(OutputSerialize, ClassWithStudents) {
    Output out;
    out.classes.push_back(Class{
        .id = parser::ClassId(2),
        .days = parser::Days(0),
        .weeks = parser::Weeks(0),
        .start = 0,
        .room = std::nullopt,
        .students =
        {
            Student{parser::StudentId(10)},
            Student{parser::StudentId(20)},
        },
    });

    OutputMetadata ctx{.name = "i",
                       .runtime = 1.0f,
                       .cores = 1,
                       .technique = "t",
                       .author = "a",
                       .institution = "i",
                       .country = "c",
                       .nr_days = 7,
                       .nr_weeks = 13};
    std::string s = out.serialize(ctx);

    EXPECT_NE(s.find(R"(<student id="10"/>)"), std::string::npos);
    EXPECT_NE(s.find(R"(<student id="20"/>)"), std::string::npos);
    EXPECT_NE(s.find("</class>"), std::string::npos);
}