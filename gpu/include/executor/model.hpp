#ifndef GPU_TIMETABLING_MODEL_HPP
#define GPU_TIMETABLING_MODEL_HPP

#include <optional>
#include <vector>

#include "parser/parser.hpp"
#include "typedefs.hpp"

struct TravelData {
    /// index into TimetableData::rooms
    usize dest_room_idx;
    u32 travel_time;

    TravelData(usize dest_room_idx, u32 travel_time);
};

struct RoomData {
    parser::RoomId id;
    u32 capacity;
    std::vector<TravelData> travels;
    std::vector<parser::TimeSlots> unavail;

    RoomData(parser::RoomId id, u32 capacity, std::vector<TravelData> travels,
             std::vector<parser::TimeSlots> unavail);
};

/// <...>_start and <...>_end are indices into the corresponding vector in
/// TimetableData, non-inclusive
struct CourseData {
    parser::CourseId id;
    usize configs_start;
    usize configs_end;

    CourseData(parser::CourseId id, usize configs_start, usize configs_end);
};

struct ConfigData {
    parser::ConfigId id;
    usize subparts_start;
    usize subparts_end;

    ConfigData(parser::ConfigId id, usize subparts_start, usize subparts_end);
};

struct SubpartData {
    parser::SubpartId id;
    usize classes_start;
    usize classes_end;

    SubpartData(parser::SubpartId id, usize classes_start, usize classes_end);
};

struct ClassData {
    parser::ClassId id;
    /// no value = no limit on number of students
    std::optional<u32> limit;
    /// no value = this class has no parent
    std::optional<usize> parent;
    /// indices into TimetableData::time_options
    usize times_start;
    usize times_end;
    /// indices into TimetableData::room_options
    /// rooms_start == rooms_end means the class doesn't need a room
    usize rooms_start;
    usize rooms_end;
    /// index into TimetableData::subparts for faster access
    usize subpart_idx;

    ClassData(parser::ClassId id, std::optional<u32> limit,
              std::optional<usize> parent, usize times_start, usize times_end,
              usize rooms_start, usize rooms_end, usize subpart_idx);

    bool needs_room() const;
};

/// describes when a class will meet
struct TimeOption {
    parser::TimeSlots times;
    u32 penalty;

    TimeOption(parser::TimeSlots times, u32 penalty);
};

/// describes where a class will meet
struct RoomOption {
    /// index into TimetableData::rooms
    usize room_idx;
    u32 penalty;

    RoomOption(usize room_idx, u32 penalty);
};

struct StudentData {
    parser::StudentId id;
    /// indices into TimetableData::courses
    std::vector<usize> course_indices;

    StudentData(parser::StudentId id, std::vector<usize> course_indices);
};

struct DistributionData {
    parser::DistributionKind kind;
    std::vector<usize> class_indices;
    /// has value = soft penalty, no value = hard penalty
    std::optional<u32> penalty;

    DistributionData(parser::DistributionKind kind,
                     std::vector<usize> class_indices,
                     std::optional<u32> penalty);

    bool is_required() const;
};

struct TimetableData {
    u32 n_days;
    u32 n_weeks;
    u32 slots_per_day;
    parser::Optimization optimization;

    std::vector<RoomData> rooms;
    std::vector<CourseData> courses;
    std::vector<ConfigData> configs;
    std::vector<SubpartData> subparts;
    std::vector<ClassData> classes;

    std::vector<TimeOption> time_options;
    std::vector<RoomOption> room_options;
    std::vector<DistributionData> distributions;
    std::vector<StudentData> students;

    static TimetableData from_problem(parser::Problem p);
};

#endif // GPU_TIMETABLING_MODEL_HPP