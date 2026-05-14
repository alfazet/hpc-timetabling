#include <thread>

#include "executor/cmd_args.hpp"
#include "executor/solver.cuh"
#include "gui.cuh"
#include "kernels/model.cuh"
#include "parser/parser.hpp"
#include "serializer/serializer.hpp"

int gui_main(int argc, char **argv) {
    WindowElements we = {};
    bool stopper = false;

    Fl::lock();

    initialize_window(&we);

    we.start_button->callback([](Fl_Widget *widget, void *data) {
        auto *we = static_cast<WindowElements *>(data);

        we->help_button->deactivate();
        we->start_button->deactivate();
        we->stop_button->activate();

        // std::thread worker_thread(timetabling_main);

        // worker_thread.detach();
    }, &we);

    we.window->end();
    we.window->show(argc, argv);

    return Fl::run();
}

template <typename lambda>
void timetabling_main(int argc, char **argv, lambda post_solver_callback, bool *stopper = nullptr) {
    ArgParser arg_parser(argc - 1, argv + 1);
    auto arg_list = arg_parser.parse_all();
    auto content = parser::utils::read_file(arg_list.dataset_path);
    auto problem = parser::Problem::parse(content);
    auto d_data = kernels::TimetableData::from_problem(problem);
    auto metadata = serializer::OutputMetadata::from_problem(problem);

    srand(arg_list.seed);
    Solver solver(d_data, arg_list.generations, arg_list.population_size, arg_list.sel_frac, arg_list.cross_rate,
                  arg_list.mut_rate, arg_list.mut_trials, arg_list.elites_frac, arg_list.worst_frac, arg_list.ls_iters,
                  arg_list.seed, stopper);
    auto best_solution = solver.solve();
    auto output = best_solution.serialize(d_data);
    auto xml = output.serialize(metadata);

    serializer::utils::write_file(arg_list.output_path, xml);
    printf("Solution file written to %s\n", arg_list.output_path.c_str());
    post_solver_callback();
}

int main(int argc, char **argv) {
    // open cli version if any argument was provided
    if (argc > 1) {
        try {
            timetabling_main(argc, argv, [] {});
        } catch (std::exception &e) {
            printf("error: %s\n", e.what());
            return 1;
        }
        return 0;
    }

    // open the gui version otherwise
    return gui_main(argc, argv);
}