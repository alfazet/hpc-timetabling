#include "kernels/model.cuh"

namespace kernels {
RoomData::RoomData(usize n_rooms, const std::vector<parser::TimeSlots> &unavail,
                   const std::vector<usize> &unavail_offsets,
                   const std::vector<u32> &travel_time,
                   const std::vector<u32> &capacity,
                   const std::vector<parser::RoomId> &id
    ) : unavail(unavail),
        unavail_offsets(unavail_offsets), travel_time(travel_time),
        capacity(capacity), id(id),
        n_rooms(n_rooms) {
}

CourseData::CourseData(const std::vector<parser::CourseId> &id,
                       const std::vector<usize> &configs_start,
                       const std::vector<usize> &configs_end) : id(id),
    configs_start(configs_start), configs_end(configs_end) {
}

ConfigData::ConfigData(const std::vector<parser::ConfigId> &id,
                       const std::vector<usize> &subparts_start,
                       const std::vector<usize> &subparts_end) : id(id),
    subparts_start(subparts_start), subparts_end(subparts_end) {
}

SubpartData::SubpartData(const std::vector<parser::SubpartId> &id,
                         const std::vector<usize> &classes_start,
                         const std::vector<usize> &classes_end) : id(id),
    classes_start(classes_start), classes_end(classes_end) {
}

ClassData::ClassData(const std::vector<parser::ClassId> &id,
                     const std::vector<u32> &limit,
                     const std::vector<usize> &parent,
                     const std::vector<usize> &times_start,
                     const std::vector<usize> &times_end,
                     const std::vector<usize> &rooms_start,
                     const std::vector<usize> &rooms_end,
                     const std::vector<usize> &subpart_idx) : id(id),
    limit(limit),
    parent(parent), times_start(times_start), times_end(times_end),
    rooms_start(rooms_start), rooms_end(rooms_end), subpart_idx(subpart_idx) {
}

TimeOption::TimeOption(const std::vector<parser::TimeSlots> &times,
                       const std::vector<u32> &penalty) : times(times),
    penalty(penalty) {
}

RoomOption::RoomOption(const std::vector<usize> &room_idx,
                       const std::vector<u32> &penalty) : room_idx(room_idx),
    penalty(penalty) {
}

TimetableData::TimetableData(u32 n_days, u32 n_weeks, u32 slots_per_day,
                             parser::Optimization optimization,
                             RoomData room_data, CourseData course_data,
                             ConfigData config_data, SubpartData subpart_data,
                             ClassData class_data, TimeOption time_options,
                             RoomOption room_options) : room_data(
        std::move(room_data)),
    courses(std::move(course_data)),
    configs(std::move(config_data)),
    subparts(std::move(subpart_data)),
    classes(std::move(class_data)),
    time_options(std::move(time_options)),
    room_options(std::move(room_options)),
    optimization(optimization),
    n_days(n_days),
    n_weeks(n_weeks),
    slots_per_day(
        slots_per_day) {
}

// TODO: split this monster
TimetableData TimetableData::from_problem(parser::Problem p) {
    std::unordered_map<usize, usize> room_id_to_idx;
    for (usize i = 0; i < p.rooms.items.size(); i++) {
        room_id_to_idx[p.rooms.items[i].id.value] = i;
    }
    usize n_rooms = p.rooms.items.size();
    std::vector<parser::TimeSlots> unavail;
    std::vector<usize> unavail_offsets;
    std::vector<u32> travel_time(n_rooms * n_rooms, NO_TRAVEL);
    std::vector<u32> capacity;
    std::vector<parser::RoomId> id;
    usize offset = 0;
    for (const auto &r : p.rooms.items) {
        usize idx = room_id_to_idx.at(r.id.value);
        unavail_offsets.push_back(offset);
        for (const auto &u : r.unavail) {
            unavail.push_back(u);
            offset++;
        }
        for (const auto &t : r.travels) {
            usize dest_idx = room_id_to_idx.at(t.room.value);
            travel_time.at(idx * n_rooms + dest_idx) = t.value;
            travel_time.at(dest_idx * n_rooms + idx) = t.value;
        }
        capacity.push_back(r.capacity);
        id.push_back(r.id);
    }
    RoomData room_data(n_rooms, unavail, unavail_offsets, travel_time,
                       capacity, id);

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
    std::vector<parser::CourseId> course_id;
    std::vector<usize> configs_start;
    std::vector<usize> configs_end;
    std::vector<parser::ConfigId> config_id;
    std::vector<usize> subparts_start;
    std::vector<usize> subparts_end;
    std::vector<parser::SubpartId> subpart_id;
    std::vector<usize> classes_start;
    std::vector<usize> classes_end;
    std::vector<parser::ClassId> class_id;
    std::vector<u32> limit;
    std::vector<usize> parent;
    std::vector<usize> times_start;
    std::vector<usize> times_end;
    std::vector<usize> rooms_start;
    std::vector<usize> rooms_end;
    std::vector<usize> subpart_idx;
    std::vector<parser::TimeSlots> times;
    std::vector<u32> time_penalty;
    std::vector<usize> room_idx;
    std::vector<u32> room_penalty;
    for (const auto &course : p.courses.items) {
        usize cfg_start = config_id.size();
        for (const auto &config : course.configs) {
            usize sbp_start = subpart_id.size();
            for (const auto &subpart : config.subparts) {
                usize cls_start = class_id.size();
                for (const auto &cls : subpart.classes) {
                    usize class_times_start = times.size();
                    for (const auto &t : cls.times) {
                        times.push_back(t.times);
                        time_penalty.push_back(t.penalty);
                    }
                    usize class_times_end = times.size();

                    usize class_rooms_start = room_idx.size();
                    for (const auto &r : cls.rooms) {
                        auto iter = room_id_to_idx.find(r.room.value);
                        if (iter != room_id_to_idx.end()) {
                            room_idx.push_back(iter->second);
                            room_penalty.push_back(r.penalty);
                        }
                    }
                    usize class_rooms_end = room_idx.size();

                    u32 class_limit = NO_LIMIT;
                    if (cls.limit.has_value()) {
                        class_limit = cls.limit.value();
                    }
                    usize parent_idx = NO_PARENT;
                    if (cls.parent.has_value()) {
                        parent_idx = class_id_to_idx.at(cls.parent->value);
                    }

                    class_id.push_back(cls.id);
                    limit.push_back(class_limit);
                    parent.push_back(parent_idx);
                    times_start.push_back(class_times_start);
                    times_end.push_back(class_times_end);
                    rooms_start.push_back(class_rooms_start);
                    rooms_end.push_back(class_rooms_end);
                    subpart_idx.push_back(subpart_id.size());
                }
                usize cls_end = class_id.size();

                subpart_id.push_back(subpart.id);
                classes_start.push_back(cls_start);
                classes_end.push_back(cls_end);
            }
            usize sbp_end = subpart_id.size();

            config_id.push_back(config.id);
            subparts_start.push_back(sbp_start);
            subparts_end.push_back(sbp_end);
        }
        usize cfg_end = config_id.size();

        course_id.push_back(course.id);
        configs_start.push_back(cfg_start);
        configs_end.push_back(cfg_end);
    }
    CourseData course_data(course_id, configs_start, configs_end);
    ConfigData config_data(config_id, subparts_start, subparts_end);
    SubpartData subpart_data(subpart_id, classes_start, classes_end);
    ClassData class_data(class_id, limit, parent, times_start, times_end,
                         rooms_start, rooms_end, subpart_idx);
    TimeOption time_options(times, time_penalty);
    RoomOption room_options(room_idx, room_penalty);

    return {p.nr_days, p.nr_weeks, p.slots_per_day, p.optimization,
            room_data, course_data, config_data, subpart_data,
            class_data, time_options, room_options};
}

}