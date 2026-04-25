#ifndef GPU_TIMETABLING_MODEL_CUH
#define GPU_TIMETABLING_MODEL_CUH

#include "common.cuh"
#include "penalty.cuh"
#include "typedefs.hpp"
#include "parser/parser.hpp"

constexpr u32 NO_TRAVEL = std::numeric_limits<u32>::max();
constexpr u32 NO_LIMIT = std::numeric_limits<u32>::max();
constexpr usize NO_PARENT = std::numeric_limits<usize>::max();

namespace kernels {

struct RoomData {
    // the unavailabilities of room `i` begin at idx unavail_offsets[i]
    // and end at idx unavail_offsets[i + 1] - 1
    // NOTE: this is an AoS, could be inefficient
    thrust::device_vector<parser::TimeSlots> unavail;
    thrust::device_vector<usize> unavail_offsets;
    // travel_time[i * n_rooms + j] = travel time between rooms `i` and `j`
    // USIZE_MAX if no travel is possible
    thrust::device_vector<u32> travel_time;
    thrust::device_vector<u32> capacity;
    thrust::device_vector<parser::RoomId> id;
    usize n_rooms;

    RoomData(usize n_rooms, const std::vector<parser::TimeSlots> &unavail,
             const std::vector<usize> &unavail_offsets,
             const std::vector<u32> &travel_time,
             const std::vector<u32> &capacity,
             const std::vector<parser::RoomId> &id
        );
};

struct CourseData {
    thrust::device_vector<parser::CourseId> id;
    thrust::device_vector<usize> configs_start;
    thrust::device_vector<usize> configs_end;

    CourseData(const std::vector<parser::CourseId> &id,
               const std::vector<usize> &configs_start,
               const std::vector<usize> &configs_end);
};

struct ConfigData {
    thrust::device_vector<parser::ConfigId> id;
    thrust::device_vector<usize> subparts_start;
    thrust::device_vector<usize> subparts_end;

    ConfigData(const std::vector<parser::ConfigId> &id,
               const std::vector<usize> &subparts_start,
               const std::vector<usize> &subparts_end);
};

struct SubpartData {
    thrust::device_vector<parser::SubpartId> id;
    thrust::device_vector<usize> classes_start;
    thrust::device_vector<usize> classes_end;

    SubpartData(const std::vector<parser::SubpartId> &id,
                const std::vector<usize> &classes_start,
                const std::vector<usize> &classes_end);
};

struct ClassData {
    thrust::device_vector<parser::ClassId> id;
    // U32_MAX if there's no limit
    thrust::device_vector<u32> limit;
    // USIZE_MAX if this class has no parent
    thrust::device_vector<usize> parent;
    // indices into TimetableData::time_options
    thrust::device_vector<usize> times_start;
    thrust::device_vector<usize> times_end;
    // indices into TimetableData::room_options
    // rooms_start == rooms_end means the class doesn't need a room
    thrust::device_vector<usize> rooms_start;
    thrust::device_vector<usize> rooms_end;
    // indices into TimetableData::subparts
    thrust::device_vector<usize> subpart_idx;

    ClassData(const std::vector<parser::ClassId> &id,
              const std::vector<u32> &limit, const std::vector<usize> &parent,
              const std::vector<usize> &times_start,
              const std::vector<usize> &times_end,
              const std::vector<usize> &rooms_start,
              const std::vector<usize> &rooms_end,
              const std::vector<usize> &subpart_idx);
};

struct TimeOption {
    thrust::device_vector<parser::TimeSlots> times;
    thrust::device_vector<u32> penalty;

    TimeOption(const std::vector<parser::TimeSlots> &times,
               const std::vector<u32> &penalty);
};

struct RoomOption {
    thrust::device_vector<usize> room_idx;
    thrust::device_vector<u32> penalty;

    RoomOption(const std::vector<usize> &room_idx,
               const std::vector<u32> &penalty);
};

struct StudentData {
    thrust::device_vector<parser::StudentId> id;
    // indices into TimetableData::courses
    thrust::device_vector<usize> course_idxs;
    // the courses wanted by student `i` begin at idx course_idxs_offsets[i]
    // and end at idx course_idxs_offsets[i + 1] - 1
    thrust::device_vector<usize> course_idxs_offsets;

    StudentData(const std::vector<parser::StudentId> &id,
                const std::vector<usize> &course_idxs,
                const std::vector<usize> &course_idxs_offsets);
};

struct DistributionData {
    // apparently CUDA supports std::variant so this should be fine
    thrust::device_vector<parser::DistributionKind> kind;
    // incides into TimetableData::classes
    thrust::device_vector<usize> class_idxs;
    // the idxs of classes taken into account by distribution `i`
    // begin at idx class_idxs_offsets[i] and end at idx class_idx_offsets[i + 1] - 1
    thrust::device_vector<usize> class_idxs_offsets;
    thrust::device_vector<Penalty> penalty;

    DistributionData(const std::vector<parser::DistributionKind> &kind,
                     const std::vector<usize> &class_idxs,
                     const std::vector<usize> &class_idxs_offsets,
                     const std::vector<Penalty> &penalty);
};

// This struct should be allocated once on the GPU's heap.
// All the kernels should just take pointers to relevant parts of this struct
// to access the problem's data.
struct TimetableData {
    RoomData room_data;
    CourseData courses;
    ConfigData configs;
    SubpartData subparts;
    ClassData classes;
    TimeOption time_options;
    RoomOption room_options;
    DistributionData distributions;
    StudentData students;

    parser::Optimization optimization;
    u32 n_days;
    u32 n_weeks;
    u32 slots_per_day;

    TimetableData(u32 n_days, u32 n_weeks, u32 slots_per_day,
                  parser::Optimization optimization,
                  RoomData room_data,
                  CourseData course_data,
                  ConfigData config_data,
                  SubpartData subpart_data,
                  ClassData class_data,
                  TimeOption time_options,
                  RoomOption room_options,
                  DistributionData distributions,
                  StudentData students
        );

    static TimetableData from_problem(parser::Problem p);
};
}

#endif // GPU_TIMETABLING_MODEL_CUH