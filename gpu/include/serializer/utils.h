#ifndef GPU_TIMETABLING_SERIALIZER_UTILS_H
#define GPU_TIMETABLING_SERIALIZER_UTILS_H

#include <string>

#include "typedefs.h"

namespace serializer::utils {
template <typename T>
std::string bit_string(T value, u32 len);

}

#endif //GPU_TIMETABLING_SERIALIZER_UTILS_H
