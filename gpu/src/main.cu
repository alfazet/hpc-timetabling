#include <FL/Fl.H>
#include <FL/Fl_Window.H>
#include <FL/Fl_Button.H>
#include <FL/Fl_Text_Buffer.H>
#include <FL/Fl_Text_Display.H>
#include <thread>

#include "executor/cmd_args.hpp"
#include "executor/solver.cuh"
#include "kernels/model.cuh"
#include "parser/parser.hpp"
#include "serializer/serializer.hpp"

int gui_main(int argc, char **argv);
void timetabling_main(int argc, char **argv);

int main(int argc, char **argv) {
    if (argc > 1) {
        try {
            timetabling_main(argc, argv);
        } catch (std::exception &e) {
            printf("error: %s\n", e.what());
            return 1;
        }
    } else {
        gui_main(argc, argv);
    }

    return 0;
}

int gui_main(int argc, char **argv) {
    Fl::lock();

    auto *window = new Fl_Window(1000, 600, "HPC Timetabling CUDA");

    auto *buffer = new Fl_Text_Buffer();
    auto *display = new Fl_Text_Display(10, 10, 480, 580);
    display->buffer(buffer);

    window->end();
    window->show(argc, argv);

    // std::thread worker_thread(background_algorithm, buffer);
    //
    // worker_thread.detach();

    return Fl::run();
}

void timetabling_main(int argc, char **argv) {
    ArgParser arg_parser(argc - 1, argv + 1);
    auto arg_list = arg_parser.parse_all();
    auto content = parser::utils::read_file(arg_list.dataset_path);
    auto problem = parser::Problem::parse(content);
    auto d_data = kernels::TimetableData::from_problem(problem);
    auto metadata = serializer::OutputMetadata::from_problem(problem);

    srand(arg_list.seed);
    Solver solver(d_data, arg_list.generations, arg_list.population_size, arg_list.sel_frac, arg_list.cross_rate,
                  arg_list.mut_rate, arg_list.mut_trials, arg_list.elites_frac, arg_list.worst_frac, arg_list.ls_iters,
                  arg_list.seed);
    auto best_solution = solver.solve();
    auto output = best_solution.serialize(d_data);
    auto xml = output.serialize(metadata);

    serializer::utils::write_file(arg_list.output_path, xml);
    printf("Solution file written to %s\n", arg_list.output_path.c_str());
}