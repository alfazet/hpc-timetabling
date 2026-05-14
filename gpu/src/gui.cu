#include "gui.cuh"

void initialize_window(WindowElements &we) {
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
}