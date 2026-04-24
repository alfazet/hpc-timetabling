#include <utility>

#include "kernels/model.cuh"

namespace kernels {
RoomData::RoomData(usize n_rooms, const std::vector<parser::TimeSlots> &unavail,
                   const std::vector<usize> &unavail_offsets,
                   const std::vector<u32> &travel_time,
                   const std::vector<u32> &capacity) : unavail(unavail),
    unavail_offsets(unavail_offsets), travel_time(travel_time),
    capacity(capacity),
    n_rooms(n_rooms) {
}

TimetableData::TimetableData(u32 n_days, u32 n_weeks, u32 slots_per_day,
                             parser::Optimization optimization,
                             RoomData room_data) : room_data(
                                                       std::move(room_data)),
                                                   optimization(optimization),
                                                   n_days(n_days),
                                                   n_weeks(n_weeks),
                                                   slots_per_day(
                                                       slots_per_day) {

}

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
    }
    RoomData room_data(n_rooms, unavail, unavail_offsets, travel_time,
                       capacity);

    TimetableData res(p.nr_days, p.nr_weeks, p.slots_per_day, p.optimization,
                      room_data);

    return res;
}

}