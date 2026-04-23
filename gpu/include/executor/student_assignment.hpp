#ifndef GPU_TIMETABLING_STUDENT_ASSIGNMENT_HPP
#define GPU_TIMETABLING_STUDENT_ASSIGNMENT_HPP

#include <vector>

#include "typedefs.hpp"

struct StudentAssignment {
    std::vector<std::vector<usize> > students_in_classes;

    // TODO; a constructor that takes in:
    // - a vector like [ students in class with id 1 .. in class with is 2 .. ]
    // - a vector of offsets into the first one, preferably scanned
    // (although the scan won't make much of a difference for this struct,
    // it could likely be useful for the evaluator kernel)
};

#endif //GPU_TIMETABLING_STUDENT_ASSIGNMENT_HPP