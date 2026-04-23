#include <gtest/gtest.h>
#include <pugixml.hpp>

#include "parser/distributions.hpp"
#include "parser/parse_error.hpp"

using namespace parser;

TEST(DistributionKind, SimpleKinds) {
    EXPECT_TRUE(
        std::holds_alternative<SameStart>(parse_distribution_kind("SameStart")
        ));
    EXPECT_TRUE(
        std::holds_alternative<SameTime>(parse_distribution_kind("SameTime")));
    EXPECT_TRUE(std::holds_alternative<DifferentTime>(
        parse_distribution_kind("DifferentTime")));
    EXPECT_TRUE(std::holds_alternative<NotOverlap>(
        parse_distribution_kind("NotOverlap")));
    EXPECT_TRUE(std::holds_alternative<SameAttendees>(
        parse_distribution_kind("SameAttendees")));
    EXPECT_TRUE(std::holds_alternative<Precedence>(
        parse_distribution_kind("Precedence")));
}

TEST(DistributionKind, SingleParameterKinds) {
    auto wd = std::get<WorkDay>(parse_distribution_kind("WorkDay(5)"));
    EXPECT_EQ(wd.s, 5);

    auto mg = std::get<MinGap>(parse_distribution_kind("MinGap(3)"));
    EXPECT_EQ(mg.g, 3);

    auto md = std::get<MaxDays>(parse_distribution_kind("MaxDays(2)"));
    EXPECT_EQ(md.d, 2);

    auto mdl = std::get<MaxDayLoad>(parse_distribution_kind("MaxDayLoad(100)"));
    EXPECT_EQ(mdl.s, 100);
}

TEST(DistributionKind, TwoParameterKinds) {
    auto mb = std::get<MaxBreaks>(parse_distribution_kind("MaxBreaks(2,10)"));
    EXPECT_EQ(mb.r, 2);
    EXPECT_EQ(mb.s, 10);

    auto mbl = std::get<MaxBlock>(parse_distribution_kind("MaxBlock(3,20)"));
    EXPECT_EQ(mbl.m, 3);
    EXPECT_EQ(mbl.s, 20);
}

TEST(DistributionKind, InvalidParameterValues) {
    EXPECT_THROW(parse_distribution_kind("MaxDays(x)"), ParseError);
    EXPECT_THROW(parse_distribution_kind("MaxBreaks(1,x)"), ParseError);
}

TEST(DistributionKind, InvalidParameterCount) {
    EXPECT_THROW(parse_distribution_kind("MaxBreaks(1)"), ParseError);
    EXPECT_THROW(parse_distribution_kind("MaxBreaks(1,2,3)"), ParseError);
}

TEST(DistributionKind, InvalidTypeName) {
    EXPECT_THROW(parse_distribution_kind("UnknownConstraint"), ParseError);
}

static pugi::xml_node load_node(pugi::xml_document &doc, const char *xml) {
    doc.load_string(xml);
    return doc.first_child();
}

TEST(Distribution, SimpleDistribution) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(
        <distributions>
            <distribution type="NotOverlap" penalty="5">
                <class id="1"/>
                <class id="2"/>
            </distribution>
        </distributions>
    )");

    auto dists = Distributions::parse(node);

    ASSERT_EQ(dists.items.size(), 1u);
    auto &dist = dists.items[0];
    EXPECT_TRUE(std::holds_alternative<NotOverlap>(dist.kind));
    EXPECT_EQ(dist.penalty, std::optional<uint32_t>(5));
    ASSERT_EQ(dist.classes.size(), 2u);
    EXPECT_EQ(dist.classes[0], ClassId::make(1));
    EXPECT_EQ(dist.classes[1], ClassId::make(2));
}

TEST(Distribution, ParameterizedDistribution) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"xml(
        <distributions>
            <distribution type="MaxDays(2)">
                <class id="3"/>
                <class id="5"/>
            </distribution>
        </distributions>
    )xml");

    auto dists = Distributions::parse(node);

    ASSERT_EQ(dists.items.size(), 1u);
    auto &dist = dists.items[0];
    auto &md = std::get<MaxDays>(dist.kind);
    EXPECT_EQ(md.d, 2);
    EXPECT_FALSE(dist.penalty.has_value());
    EXPECT_EQ(dist.classes.size(), 2u);
}

TEST(Distribution, MissingType) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(
        <distributions>
            <distribution penalty="3">
                <class id="1"/>
            </distribution>
        </distributions>
    )");

    EXPECT_THROW(Distributions::parse(node), ParseError);
}

TEST(Distribution, ClassUnexpectedAttribute) {
    pugi::xml_document doc;
    auto node = load_node(doc, R"(
        <distributions>
            <distribution type="NotOverlap">
                <class id="1" foo="bar"/>
            </distribution>
        </distributions>
    )");

    EXPECT_THROW(Distributions::parse(node), ParseError);
}
