#ifndef GPU_TIMETABLING_PROBLEM_H
#define GPU_TIMETABLING_PROBLEM_H

#include <string>

#include "typedefs.hpp"
#include "courses.hpp"
#include "distributions.hpp"
#include "optimization.hpp"
#include "rooms.hpp"
#include "students.hpp"

namespace parser {
struct Problem {
    std::string name;
    u32 nr_days;
    u32 nr_weeks;
    u32 slots_per_day;

    Optimization optimization;
    Rooms rooms;
    Courses courses;
    Distributions distributions;
    Students students;

    static Problem parse(const std::string &xml);
};

}

#endif //GPU_TIMETABLING_PROBLEM_H