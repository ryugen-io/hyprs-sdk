// Memory management bridge functions.
#include "common.h"

extern "C" void hyprland_api_free_string(char* ptr) {
    std::free(ptr);
}

extern "C" void hyprland_api_free_array(void* ptr) {
    std::free(ptr);
}
