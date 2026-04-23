#ifndef GPU_TIMETABLING_PARSE_ERROR_H
#define GPU_TIMETABLING_PARSE_ERROR_H

#include <stdexcept>
#include <string>

namespace parser {
class ParseError : public std::runtime_error {
public:
    using std::runtime_error::runtime_error;

    static ParseError missing_attr(const char *name) {
        return ParseError(std::string("Missing attribute `") + name + "`");
    }

    static ParseError missing_element(const char *name) {
        return ParseError(std::string("Missing element `") + name + "`");
    }

    static ParseError unexpected_attr(const std::string &name) {
        return ParseError(std::string("Unexpected attribute `") + name + "`");
    }

    static ParseError
    invalid_value(const char *attr, const std::string &value) {
        return ParseError(
            std::string("Invalid value for `") + attr + "`: `" + value + "`");
    }

    static ParseError invalid_bitstring(const std::string &name) {
        return ParseError(std::string("Invalid bit string for `") + name + "`");
    }
};

}

#endif //GPU_TIMETABLING_PARSE_ERROR_H