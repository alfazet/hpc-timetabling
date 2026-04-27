#include "executor/solver.cuh"

#include "kernels/assigner.cuh"
#include "kernels/model.cuh"
#include "kernels/population.cuh"

FoundSolution::FoundSolution(
    std::vector<std::vector<usize> > student_assignment,
    std::vector<parser::TimeSlots> times,
    std::vector<usize> rooms_idxs,
    std::pair<u32, u32> penalty)
    : student_assignment(std::move(student_assignment)),
      times(std::move(times)), rooms_idxs(std::move(rooms_idxs)),
      penalty(std::move(penalty)) {
}

serializer::Output
FoundSolution::serialize(const std::vector<parser::RoomId> &room_ids,
                         const std::vector<parser::StudentId> &student_ids,
                         const std::vector<parser::ClassId> &class_ids) const {
    std::vector<serializer::Class> classes_out;
    for (usize i = 0; i < class_ids.size(); i++) {
        usize room_idx = this->rooms_idxs[i];
        auto room = room_idx == NO_ROOM
                        ? std::optional<parser::RoomId>()
                        : room_ids[room_idx];
        auto time = this->times[i];

        std::vector<serializer::Student> students;
        for (usize student_idx : this->student_assignment[i]) {
            students.emplace_back(student_ids[student_idx]);
        }

        classes_out.emplace_back(class_ids[i], time.days, time.weeks,
                                 time.start, room, students);
    }

    return {classes_out};
}

Solver::Solver(
    kernels::TimetableData d_data, u32 generations, u32 population_size,
    u32 seed)
    : d_data(std::move(d_data)), generations(generations),
      population_size(population_size),
      seed(seed) {
}

FoundSolution Solver::solve() const {
    usize n_classes = d_data.classes.id.size();

    kernels::Population
        population(n_classes, this->population_size, this->seed);
    population.init(d_data);

    kernels::StudentAssignment assignment(n_classes, this->population_size);
    assignment.assign(d_data, population);

    // TODO: main loop over generations

    // TODO: create a FoundSolution

    exit(123);
}