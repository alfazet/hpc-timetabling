#ifndef GPU_TIMETABLING_MODEL_CUH
#define GPU_TIMETABLING_MODEL_CUH

#include "parser/parser.hpp"
#include "typedefs.hpp"

#include <thrust/device_vector.h>

struct TravelData {
    /// index into TimetableData::rooms
    usize dest_room_idx;
    u32 travel_time;

    TravelData(usize dest_room_idx, u32 travel_time);
};

struct RoomData {
    parser::RoomId id;
    u32 capacity;
    thrust::device_vector<TravelData> travels_;
    TravelData *travels;
    usize travels_count;
    thrust::device_vector<parser::TimeSlots> unavail_;
    parser::TimeSlots *unavail;
    usize unavail_count;

    RoomData(parser::RoomId id, u32 capacity, std::vector<TravelData> &travels,
             std::vector<parser::TimeSlots> &unavail);
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
    thrust::device_vector<usize> course_indices_;
    usize *course_indices;
    usize course_indices_count;

    StudentData(parser::StudentId id, std::vector<usize> &course_indices);
};

struct DistributionData {
    parser::DistributionKind kind;
    thrust::device_vector<usize> class_indices_;
    usize *class_indices;
    usize class_indices_count;
    /// has value = soft penalty, no value = hard penalty
    std::optional<u32> penalty;

    DistributionData(parser::DistributionKind kind,
                     std::vector<usize> &class_indices,
                     std::optional<u32> penalty);

    bool is_required() const;
};

struct TimetableData {
    u32 n_days;
    u32 n_weeks;
    u32 slots_per_day;
    parser::Optimization optimization;

    thrust::device_vector<RoomData> rooms_;
    RoomData *rooms;
    usize rooms_count;
    thrust::device_vector<CourseData> courses_;
    CourseData *courses;
    usize courses_count;
    thrust::device_vector<ConfigData> configs_;
    ConfigData *configs;
    usize configs_count;
    thrust::device_vector<SubpartData> subparts_;
    SubpartData *subparts;
    usize subparts_count;
    thrust::device_vector<ClassData> classes_;
    ClassData *classes;
    usize classes_count;

    thrust::device_vector<TimeOption> time_options_;
    TimeOption *time_options;
    usize time_options_count;
    thrust::device_vector<RoomOption> room_options_;
    RoomOption *room_options;
    usize room_options_count;
    thrust::device_vector<DistributionData> distributions_;
    DistributionData *distributions;
    usize distributions_count;
    thrust::device_vector<StudentData> students_;
    StudentData *students;
    usize students_count;

    static TimetableData from_problem(parser::Problem p);
};

#endif // GPU_TIMETABLING_MODEL_CUH