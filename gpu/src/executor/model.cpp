#include "executor/model.hpp"

#include <unordered_map>

TravelData::TravelData(usize dest_room_idx, u32 travel_time)
    : dest_room_idx(dest_room_idx), travel_time(travel_time) {
}

RoomData::RoomData(parser::RoomId id, u32 capacity,
                   std::vector<TravelData> travels,
                   std::vector<parser::TimeSlots> unavail)
    : id(id), capacity(capacity), travels(std::move(travels)),
      unavail(std::move(unavail)) {
}

CourseData::CourseData(parser::CourseId id, usize configs_start,
                       usize configs_end)
    : id(id), configs_start(configs_start), configs_end(configs_end) {
}

ConfigData::ConfigData(parser::ConfigId id, usize subparts_start,
                       usize subparts_end)
    : id(id), subparts_start(subparts_start), subparts_end(subparts_end) {
}

SubpartData::SubpartData(parser::SubpartId id, usize classes_start,
                         usize classes_end)
    : id(id), classes_start(classes_start), classes_end(classes_end) {
}

ClassData::ClassData(parser::ClassId id, std::optional<u32> limit,
                     std::optional<usize> parent, usize times_start,
                     usize times_end, usize rooms_start, usize rooms_end,
                     usize subpart_idx)
    : id(id), limit(limit), parent(parent), times_start(times_start),
      times_end(times_end), rooms_start(rooms_start), rooms_end(rooms_end),
      subpart_idx(subpart_idx) {
}

bool ClassData::needs_room() const { return rooms_start != rooms_end; }

TimeOption::TimeOption(parser::TimeSlots times, u32 penalty)
    : times(times), penalty(penalty) {
}

RoomOption::RoomOption(usize room_idx, u32 penalty)
    : room_idx(room_idx), penalty(penalty) {
}

StudentData::StudentData(parser::StudentId id,
                         std::vector<usize> course_indices)
    : id(id), course_indices(std::move(course_indices)) {
}

DistributionData::DistributionData(parser::DistributionKind kind,
                                   std::vector<usize> class_indices,
                                   std::optional<u32> penalty)
    : kind(kind), class_indices(std::move(class_indices)),
      penalty(penalty) {
}

bool DistributionData::is_required() const { return !penalty.has_value(); }

TimetableData TimetableData::from_problem(parser::Problem p) {
    std::unordered_map<usize, usize> room_id_to_idx;
    for (usize i = 0; i < p.rooms.items.size(); i++) {
        room_id_to_idx[p.rooms.items[i].id.value] = i;
    }
    std::unordered_map<usize, usize> course_id_to_idx;
    for (usize i = 0; i < p.courses.items.size(); i++) {
        course_id_to_idx[p.courses.items[i].id.value] = i;
    }

    std::unordered_map<usize, usize> class_id_to_idx;
    usize idx = 0;
    for (const auto &course : p.courses.items) {
        for (const auto &config : course.configs) {
            for (const auto &subpart : config.subparts) {
                for (const auto &cls : subpart.classes) {
                    class_id_to_idx[cls.id.value] = idx++;
                }
            }
        }
    }

    std::vector<RoomData> rooms;
    rooms.reserve(p.rooms.items.size());
    for (auto &r : p.rooms.items) {
        std::vector<TravelData> travels;
        travels.reserve(r.travels.size());
        for (const auto &t : r.travels) {
            travels.emplace_back(room_id_to_idx.at(t.room.value), t.value);
        }
        rooms.emplace_back(r.id, r.capacity, std::move(travels),
                           r.unavailabilities);
    }

    std::vector<StudentData> students;
    students.reserve(p.students.items.size());
    for (auto &s : p.students.items) {
        std::vector<usize> course_indices;
        for (const auto &cid : s.courses) {
            auto it = course_id_to_idx.find(cid.value);
            if (it != course_id_to_idx.end()) {
                course_indices.push_back(it->second);
            }
        }
        students.emplace_back(s.id, std::move(course_indices));
    }

    std::vector<DistributionData> distributions;
    distributions.reserve(p.distributions.items.size());
    for (auto &d : p.distributions.items) {
        std::vector<usize> class_indices;
        for (const auto &cid : d.classes) {
            auto it = class_id_to_idx.find(cid.value);
            if (it != class_id_to_idx.end()) {
                class_indices.push_back(it->second);
            }
        }
        distributions.emplace_back(d.kind, std::move(class_indices),
                                   d.penalty);
    }

    std::vector<CourseData> courses;
    std::vector<ConfigData> configs;
    std::vector<SubpartData> subparts;
    std::vector<ClassData> classes;
    std::vector<TimeOption> time_options;
    std::vector<RoomOption> room_options;
    for (auto &course : p.courses.items) {
        usize configs_start = configs.size();

        for (auto &config : course.configs) {
            usize subparts_start = subparts.size();

            for (auto &subpart : config.subparts) {
                usize classes_start = classes.size();

                for (auto &cls : subpart.classes) {
                    usize times_start = time_options.size();
                    for (auto &t : cls.times) {
                        time_options.emplace_back(t.times, t.penalty);
                    }
                    usize times_end = time_options.size();

                    usize rooms_start = room_options.size();
                    for (const auto &r : cls.rooms) {
                        auto it = room_id_to_idx.find(r.room.value);
                        if (it != room_id_to_idx.end()) {
                            room_options.emplace_back(it->second, r.penalty);
                        }
                    }
                    usize rooms_end = room_options.size();

                    std::optional<usize> parent_idx;
                    if (cls.parent.has_value()) {
                        parent_idx = class_id_to_idx.at(cls.parent->value);
                    }

                    classes.emplace_back(cls.id, cls.limit, parent_idx,
                                         times_start, times_end, rooms_start,
                                         rooms_end, subparts.size());
                }

                usize classes_end = classes.size();
                subparts.emplace_back(subpart.id, classes_start, classes_end);
            }

            usize subparts_end = subparts.size();
            configs.emplace_back(config.id, subparts_start, subparts_end);
        }

        usize configs_end = configs.size();
        courses.emplace_back(course.id, configs_start, configs_end);
    }

    TimetableData result;
    result.n_days = p.nr_days;
    result.n_weeks = p.nr_weeks;
    result.slots_per_day = p.slots_per_day;
    result.optimization = p.optimization;
    result.rooms = std::move(rooms);
    result.courses = std::move(courses);
    result.configs = std::move(configs);
    result.subparts = std::move(subparts);
    result.classes = std::move(classes);
    result.time_options = std::move(time_options);
    result.room_options = std::move(room_options);
    result.distributions = std::move(distributions);
    result.students = std::move(students);

    return result;
}