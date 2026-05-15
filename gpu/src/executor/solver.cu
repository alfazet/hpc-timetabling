#include "executor/solver.cuh"

#include <iomanip>

#include "executor/adjuster.cuh"
#include "executor/timer.cuh"
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
            students.push_back({student_ids[student_idx]});
        }

        classes_out.push_back({class_ids[i], time.days, time.weeks, time.start, room, students});
    }

    return {classes_out};
}

Solver::Solver(kernels::TimetableData d_data, u32 generations, u32 population_size, f32 sel_frac, f32 cross_rate,
               f32 mut_rate, u32 mut_trials, f32 elites_frac, f32 worst_frac, u32 ls_iters, u32 tournament_size,
               u32 seed, bool *stopper)
    : d_data(std::move(d_data)), generations(generations), population_size(population_size), sel_frac(sel_frac),
      cross_rate(cross_rate), mut_rate(mut_rate), mut_trials(mut_trials), elites_frac(elites_frac),
      worst_frac(worst_frac), ls_iters(ls_iters), tournament_size(tournament_size), seed(seed), stopper(stopper) {}

void Solver::print_metadata(std::ostream &out) const {
    out << "Solver started...\n"
        << "Generations: " << generations << "\n"
        << "Population size: " << population_size << "\n"
        << "Selection: " << std::fixed << std::setprecision(1) << (sel_frac * 100.0) << "%\n"
        << std::setprecision(4) << "Crossover rate: " << cross_rate << "\n"
        << "Mutation rate: " << mut_rate << "\n"
        << "Mutation trials per iter: " << mut_trials << "\n"
        << "Elites: " << (elites_frac * 100.0) << "%\n"
        << "Anti-elites: " << (worst_frac * 100.0) << "%\n"
        << "Local search iterations: " << ls_iters << "\n"
        << "Tournament size: " << tournament_size << "\n"
        << "Seed: " << seed << "\n";
}

FoundSolution Solver::solve(std::ostream &out) const {
    usize n_classes = d_data.classes.id.size();
    usize n_students = d_data.students.id.size();

    f32 delta = 0.05;
    f32 min_mut = 0.1, max_mut = 0.9;
    f32 min_cross = 0.1, max_cross = 0.9;
    f32 min_elites_frac = 0.05, max_elites_frac = 0.05;
    f32 min_worst_frac = 0.05, max_worst_frac = 0.25;
    Adjuster adjuster(delta, min_mut, max_mut, min_cross, max_cross, min_elites_frac, max_elites_frac, min_worst_frac,
                      max_worst_frac);
    Stats stats;

    kernels::Evaluator evaluator;
    kernels::Population population(n_students, n_classes, this->population_size, this->elites_frac, this->worst_frac,
                                   this->seed);
    kernels::StudentAssignment assignment(n_classes, this->population_size);
    kernels::Crossover crossover(this->cross_rate);
    kernels::Mutation mutation(this->mut_rate, this->mut_trials);
    kernels::Selection selection(this->population_size, this->sel_frac, this->tournament_size);
    kernels::LocalSearch local_search(this->ls_iters);
    population.init(d_data);

    this->print_metadata(out);
    u32 update_interval = (generations + 100 - 1) / 100;
    Timer timer;
    for (u32 gen = 1; gen <= generations; gen++) {
        timer.start();
        local_search.search(population, d_data);
        assignment.assign(d_data, population);
        evaluator.evaluate(d_data, population, assignment);
        population.sort();

        // TODO: the penalties could likely be cached and reused
        population.replace_worst(d_data, mutation.prob / 2);
        local_search.search(population, d_data);
        assignment.assign(d_data, population);
        evaluator.evaluate(d_data, population, assignment);
        population.sort();
        timer.stop();

        if (gen % update_interval == 0) {
            stats.update(gen, population.get_best_penalty());
            adjuster.adjust(stats, mutation, crossover, population);
            stats.print(mutation.prob, crossover.prob, population.elites_frac, population.worst_frac, out);
        }

        timer.start();
        selection.select(population);
        crossover.next_population(selection, population, d_data);
        mutation.apply_mutations(population, d_data);
        timer.stop();
        if (gen % update_interval == 0) {
            timer.print(update_interval, out);
            timer = {};
        }

        if (this->stopper && *this->stopper) {
            break;
        }
    }

    return population.get_best_solution(assignment);
}
