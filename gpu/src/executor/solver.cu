#include "executor/solver.cuh"

#include "executor/adjuster.cuh"
#include "kernels/assigner.cuh"
#include "kernels/crossover.cuh"
#include "kernels/evaluator.cuh"
#include "kernels/local_search.cuh"
#include "kernels/model.cuh"
#include "kernels/mutation.cuh"
#include "kernels/penalty.cuh"
#include "kernels/population.cuh"
#include "kernels/selection.cuh"

FoundSolution::FoundSolution(std::vector<std::vector<u16>> student_assignment, std::vector<u16> times_idxs,
                             std::vector<u16> rooms_idxs, kernels::Penalty penalty)
    : student_assignment(std::move(student_assignment)), times_idxs(std::move(times_idxs)),
      rooms_idxs(std::move(rooms_idxs)), penalty(penalty) {}

serializer::Output FoundSolution::serialize(const kernels::TimetableData &d_data) const {
    std::vector<serializer::Class> classes_out;
    auto class_ids = d_data.get_class_ids();
    auto room_ids = d_data.get_room_ids();
    auto student_ids = d_data.get_student_ids();
    auto time_slots = d_data.get_time_slots();
    for (usize i = 0; i < class_ids.size(); i++) {
        u16 room_idx = this->rooms_idxs[i];
        auto room = room_idx == NO_ROOM ? std::optional<parser::RoomId>() : room_ids[room_idx];

        u16 time_idx = this->times_idxs[i];
        auto time = time_slots[time_idx];

        std::vector<serializer::Student> students;
        for (u16 student_idx : this->student_assignment[i]) {
            students.emplace_back(student_ids[student_idx]);
        }

        classes_out.emplace_back(class_ids[i], time.days, time.weeks, time.start, room, students);
    }

    return {classes_out};
}

Solver::Solver(kernels::TimetableData d_data, u32 generations, u32 population_size, f32 sel_frac, f32 cross_rate,
               f32 mut_rate, f32 elites_frac, u32 ls_iters, u32 ls_trials, u32 seed)
    : d_data(std::move(d_data)), generations(generations), population_size(population_size), sel_frac(sel_frac),
      cross_rate(cross_rate), mut_rate(mut_rate), elites_frac(elites_frac), ls_iters(ls_iters), ls_trials(ls_trials),
      seed(seed) {}

void Solver::print_metadata() const {
    printf("Solver started...\n");
    printf("Generations: %u\n", generations);
    printf("Population size: %u\n", population_size);
    printf("Selection: %.1f%%\n", sel_frac * 100);
    printf("Crossover rate: %.4f\n", cross_rate);
    printf("Elites: %.1f%%\n", elites_frac * 100);
    printf("Local search iterations: %u\n", ls_iters);
    printf("Local search trials per iter: %u\n", ls_trials);
    printf("Seed: %u\n", seed);
}

FoundSolution Solver::solve() const {
    usize n_classes = d_data.classes.id.size();

    Adjuster adjuster(0.05, 0.1, 0.9, 0.1, 0.9);
    Stats stats;

    kernels::Evaluator evaluator;
    kernels::Population population(n_classes, this->population_size, this->elites_frac, this->seed);
    kernels::StudentAssignment assignment(n_classes, this->population_size);
    kernels::Crossover crossover(this->cross_rate);
    kernels::Mutation mutation(this->mut_rate);
    kernels::Selection selection(this->population_size, this->sel_frac);
    kernels::LocalSearch local_search(this->ls_iters, this->ls_trials);
    population.init(d_data);

    this->print_metadata();
    FoundSolution sol = population.get_best_solution(assignment);
    for (u32 gen = 1; gen <= generations; gen++) {
        local_search.search(population, d_data);
        assignment.assign(d_data, population);
        evaluator.evaluate(d_data, population, assignment);
        population.sort();

        if (gen % ((generations + 100 - 1) / 100) == 0) {
            sol = population.get_best_solution(assignment);
            stats.update(gen, sol.penalty);
            adjuster.adjust(stats, mutation, crossover);
            stats.print();
        }

        selection.select(population);
        crossover.next_population(selection, population);
        mutation.apply_mutations(population, d_data);
    }

    return sol;
}
