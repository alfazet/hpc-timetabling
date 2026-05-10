#ifndef GPU_TIMETABLING_TIMER_CUH
#define GPU_TIMETABLING_TIMER_CUH

#include "typedefs.hpp"

class Timer {
  public:
    float elapsed = 0; // in ms

    void start();

    void stop();

    void print(u32 generations);

  private:
    cudaEvent_t start_event{};
    cudaEvent_t stop_event{};
};

#endif // GPU_TIMETABLING_TIMER_CUH
