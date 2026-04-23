#include <gtest/gtest.h>

#include "parser/parse_error.hpp"
#include "parser/days.hpp"

using namespace parser;

TEST(Days, Bitstring) {
    auto days = Days::parse("110000");
    EXPECT_EQ(days.bits, (1 << 0) | (1 << 1));
    for (int i = 0; i < 2; ++i)
        EXPECT_TRUE(days.contains(i));
    for (int i = 2; i < 7; ++i)
        EXPECT_FALSE(days.contains(i));
}

TEST(Days, AllZero) {
    auto days = Days::parse("0000000");
    for (int d = 0; d < 7; ++d)
        EXPECT_FALSE(days.contains(d));
}

TEST(Days, AllOne) {
    auto days = Days::parse("1111111");
    for (int d = 0; d < 7; ++d)
        EXPECT_TRUE(days.contains(d));
}

TEST(Days, InvalidCharacter) {
    EXPECT_THROW(Days::parse("10a0000"), ParseError);
}

TEST(Days, TooLong) { EXPECT_THROW(Days::parse("10000000"), ParseError); }