#include <FL/Fl.H>
#include <FL/Fl_Box.H>
#include <FL/Fl_Button.H>
#include <FL/Fl_Text_Editor.H>
#include <FL/Fl_Text_Buffer.H>
#include <FL/Fl_Text_Display.H>
#include <FL/Fl_Window.H>
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
        return 0;
    }

    return gui_main(argc, argv);
}

struct WindowElements {
    Fl_Window *window;
    Fl_Text_Buffer *logs_buffer;
    Fl_Text_Display *logs_display;
    Fl_Box *commands_label;
    Fl_Text_Buffer *commands_buffer;
    Fl_Text_Editor *commands_input;
    Fl_Button *help_button;
    Fl_Button *start_button;
    Fl_Button *stop_button;
    Fl_Box *information_label;
};

int gui_main(int argc, char **argv) {
    WindowElements we = {};

    Fl::lock();

    we.window = new Fl_Window(1000, 600, "HPC Timetabling CUDA");

    we.logs_buffer = new Fl_Text_Buffer();
    we.logs_display = new Fl_Text_Display(10, 10, 480, 580);
    we.logs_display->buffer(we.logs_buffer);

    we.commands_label = new Fl_Box(510, 10, 480, 20, "Commands:");
    we.commands_buffer = new Fl_Text_Buffer();
    we.commands_input = new Fl_Text_Editor(510, 30, 480, 80);
    we.commands_input->buffer(we.commands_buffer);
    we.commands_input->wrap_mode(Fl_Text_Editor::WRAP_AT_BOUNDS, 0);

    we.help_button = new Fl_Button(640, 120, 110, 40, "Help...");

    we.start_button = new Fl_Button(760, 120, 110, 40, "Start!");

    we.stop_button = new Fl_Button(880, 120, 110, 40, "Stop!");
    we.stop_button->deactivate();

    we.information_label = new Fl_Box(510, 170, 480, 420, "Tutaj pojawia sie informacje");

    we.window->end();
    we.window->show(argc, argv);

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