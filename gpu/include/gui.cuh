#ifndef GPU_TIMETABLING_GUI_CUH
#define GPU_TIMETABLING_GUI_CUH

#include <FL/Fl.H>
#include <FL/Fl_Window.H>
#include <FL/Fl_Box.H>
#include <FL/Fl_Button.H>
#include <FL/Fl_Text_Buffer.H>
#include <FL/Fl_Text_Display.H>
#include <FL/Fl_Text_Editor.H>
#include <string>
#include <vector>

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
    bool stopper = false;
};

void initialize_window(WindowElements *we);
void help_callback(Fl_Widget *widget, void *data);
std::vector<std::string> parse_cmdline(const std::string &cmd);

#endif // GPU_TIMETABLING_GUI_CUH
