#include "serializer/utils.h"

namespace serializer::utils {
template <typename T>
std::string bit_string(T value, u32 len) {
    std::string s;
    s.reserve(len);
    for (u32 i = 0; i < len; i++)
        s.push_back((value & (1u << i)) ? '1' : '0');

    return s;
}

template std::string bit_string<u8>(u8, u32);

template std::string bit_string<u16>(u16, u32);

template std::string bit_string<u32>(u32, u32);

}
