#include "kernels/model.cuh"

TravelData::TravelData(usize dest_room_idx, u32 travel_time)
    : dest_room_idx(dest_room_idx), travel_time(travel_time) {
}

RoomData::RoomData(parser::RoomId id, u32 capacity,
                   std::vector<TravelData> &travels,
                   std::vector<parser::TimeSlots> &unavail)
    : id(id), capacity(capacity), travels_(travels), travels(travels_.data().get()),
      travels_count(travels.size()), unavail_(unavail),
      unavail(unavail_.data().get()), unavail_count(unavail.size()) {}

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
                         std::vector<usize> &course_indices)
    : id(id), course_indices_(course_indices),
      course_indices(course_indices_.data().get()),
      course_indices_count(course_indices.size()) {
}

DistributionData::DistributionData(parser::DistributionKind kind,
                                   std::vector<usize> &class_indices,
                                   std::optional<u32> penalty)
    : kind(kind), class_indices_(class_indices),
      class_indices(class_indices_.data().get()),
      class_indices_count(class_indices.size()),
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

    result.rooms_ = rooms;
    result.rooms = result.rooms_.data().get();
    result.rooms_count = rooms.size();

    result.courses_ = courses;
    result.courses = result.courses_.data().get();
    result.courses_count = courses.size();

    result.configs_ = configs;
    result.configs = result.configs_.data().get();
    result.configs_count = configs.size();

    result.subparts_ = subparts;
    result.subparts = result.subparts_.data().get();
    result.subparts_count = subparts.size();

    result.classes_ = classes;
    result.classes = result.classes_.data().get();
    result.classes_count = classes.size();

    result.time_options_ = time_options;
    result.time_options = result.time_options_.data().get();
    result.time_options_count = time_options.size();

    result.room_options_ = room_options;
    result.room_options = result.room_options_.data().get();
    result.room_options_count = room_options.size();

    result.distributions_ = distributions;
    result.distributions = result.distributions_.data().get();
    result.distributions_count = distributions.size();

    result.students_ = students;
    result.students = result.students_.data().get();
    result.students_count = students.size();

    return result;
}