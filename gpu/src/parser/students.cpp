#include <pugixml.hpp>

#include "parser/utils.hpp"
#include "parser/parse_error.hpp"
#include "parser/students.hpp"

namespace parser {

Students Students::parse(const pugi::xml_node &node) {
    if (node.first_attribute())
        throw ParseError::unexpected_attr("students");

    Students result;
    for (auto student_node : node.children("student")) {
        utils::reject_extra_attrs(student_node, {"id"});

        Student student;
        student.id = StudentId(utils::required_int<u32>(student_node, "id"));
        for (auto course_node : student_node.children("course")) {
            utils::reject_extra_attrs(course_node, {"id"});
            student.courses.push_back(
                CourseId::make(utils::required_int<u32>(course_node, "id")));
        }
        result.items.push_back(std::move(student));
    }

    return result;
}

}