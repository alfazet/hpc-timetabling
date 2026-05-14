#include "gui.cuh"

#include "executor/cmd_args.hpp"

std::streamsize FlBufferStreamBuf::xsputn(const char *s, std::streamsize count) {
    std::string text(s, count);

    Fl::lock();
    fl_buf->append(text.c_str());
    for (auto display : fl_displays) {
        display->insert_position(fl_buf->length());
        display->show_insert_position();
    }
    Fl::awake();
    Fl::unlock();

    return count;
}

std::streambuf::int_type FlBufferStreamBuf::overflow(int_type c) {
    if (c != traits_type::eof()) {
        char ch = traits_type::to_char_type(c);
        char str[2] = {ch, '\0'};

        Fl::lock();
        fl_buf->append(str);
        for (auto display : fl_displays) {
            display->insert_position(fl_buf->length());
            display->show_insert_position();
        }
        Fl::awake();
        Fl::unlock();
    }
    return c;
}

FlBufferStreamBuf::FlBufferStreamBuf(Fl_Text_Buffer *buf, std::vector<Fl_Text_Display *> &fl_displays)
    : fl_buf(buf), fl_displays(fl_displays) {}

FlBufferStream::FlBufferStream(Fl_Text_Buffer *fl_buf, std::vector<Fl_Text_Display *> &fl_displays)
    : std::ostream(&buf), buf(fl_buf, fl_displays) {}

void initialize_window(WindowElements *we) {
    we->window = new Fl_Window(1000, 600, "HPC Timetabling CUDA");

    we->logs_buffer = new Fl_Text_Buffer();
    we->logs_display = new Fl_Text_Display(10, 10, 480, 580);
    we->logs_display->buffer(we->logs_buffer);
    std::vector displays = {we->logs_display};
    we->logs_buffer_stream = new FlBufferStream(we->logs_buffer, displays);

    we->commands_label = new Fl_Box(510, 10, 480, 20, "Commands:");
    we->commands_buffer = new Fl_Text_Buffer();
    we->commands_input = new Fl_Text_Editor(510, 30, 480, 80);
    we->commands_input->buffer(we->commands_buffer);
    we->commands_input->wrap_mode(Fl_Text_Editor::WRAP_AT_BOUNDS, 0);

    we->help_button = new Fl_Button(640, 120, 110, 40, "Help...");
    we->help_button->callback(help_callback, we);

    we->start_button = new Fl_Button(760, 120, 110, 40, "Start!");

    we->stop_button = new Fl_Button(880, 120, 110, 40, "Stop!");
    we->stop_button->deactivate();

    we->information_label = new Fl_Box(510, 170, 480, 420);
    we->information_label->align(FL_ALIGN_LEFT | FL_ALIGN_INSIDE);
}

void help_callback(Fl_Widget *widget, void *data) {
    auto *we = static_cast<WindowElements *>(data);

    we->information_label->label(
        "List of all commands:\n"
        #define X(flag, field, type, parser, default_val, help) flag ": " help "\n"
        ARG_TABLE(X)
        #undef X
    );
}

std::vector<std::string> parse_cmdline(const std::string &cmd) {
    std::vector<std::string> args;
    std::string current_arg;
    bool in_quotes = false;

    for (char c : cmd) {
        if (c == '\"') {
            in_quotes = !in_quotes;
        } else if (c == ' ' && !in_quotes) {
            if (!current_arg.empty()) {
                args.push_back(current_arg);
                current_arg.clear();
            }
        } else {
            current_arg += c;
        }
    }

    if (!current_arg.empty()) {
        args.push_back(current_arg);
    }

    return args;
}