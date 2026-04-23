#include <gtest/gtest.h>

#include "parser/weeks.hpp"

using namespace parser;

TEST(Weeks, Bitstring) {
    auto w = Weeks::parse("1100000");
    EXPECT_EQ(w.bits, (1 << 0) | (1 << 1));
    for (int i = 0; i < 2; ++i)
        EXPECT_TRUE(w.contains(i));
    for (int i = 2; i < 7; ++i)
        EXPECT_FALSE(w.contains(i));
}

TEST(Weeks, AllZero) {
    auto w = Weeks::parse("0000000");
    for (int d = 0; d < 7; ++d)
        EXPECT_FALSE(w.contains(d));
}

TEST(Weeks, AllOne) {
    auto w = Weeks::parse("1111111");
    for (int d = 0; d < 7; ++d)
        EXPECT_TRUE(w.contains(d));
}

TEST(Weeks, InvalidCharacter) {
    EXPECT_THROW(Weeks::parse("10a0000"), ParseError);
}
