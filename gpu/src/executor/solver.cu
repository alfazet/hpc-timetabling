#include "executor/solver.cuh"

#include "kernels/assigner.cuh"
#include "kernels/evaluator.cuh"
#include "kernels/model.cuh"
#include "kernels/population.cuh"

FoundSolution::FoundSolution(
    std::vector<std::vector<u16> > student_assignment,
    std::vector<u16> times_idxs,
    std::vector<u16> rooms_idxs,
    std::pair<u32, u32> penalty)
    : student_assignment(std::move(student_assignment)),
      times_idxs(std::move(times_idxs)), rooms_idxs(std::move(rooms_idxs)),
      penalty(std::move(penalty)) {
}

serializer::Output
FoundSolution::serialize(const std::vector<parser::RoomId> &room_ids,
                         const std::vector<parser::StudentId> &student_ids,
                         const std::vector<parser::ClassId> &class_ids,
                         const std::vector<parser::TimeSlots> &time_slots) const {
    std::vector<serializer::Class> classes_out;
    for (usize i = 0; i < class_ids.size(); i++) {
        u16 room_idx = this->rooms_idxs[i];
        auto room = room_idx == NO_ROOM
                        ? std::optional<parser::RoomId>()
                        : room_ids[room_idx];

        u16 time_idx = this->times_idxs[i];
        auto time = time_slots[time_idx];

        std::vector<serializer::Student> students;
        for (u16 student_idx : this->student_assignment[i]) {
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

    for (u32 gen = 1; gen <= generations; gen++) {
        printf("assignment start\n");
        assignment.assign(d_data, population);
        printf("evaluation start\n");
        kernels::evaluator::evaluate(d_data, population, assignment);
    }

    return population.get_best_solution(assignment);
}