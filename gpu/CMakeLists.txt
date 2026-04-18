cmake_minimum_required(VERSION 4.0)
project(gpu-timetabling LANGUAGES CXX CUDA)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CUDA_STANDARD 20)
set(CMAKE_CUDA_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
include(FetchContent)

include_directories(include)

file(GLOB_RECURSE PARSER_SRC src/parser/*.cpp include/parser/*.h)
add_library(parser ${PARSER_SRC})

file(GLOB_RECURSE SERIALIZER_SRC src/serializer/*.cpp include/serializer/*.h)
add_library(serializer ${SERIALIZER_SRC})
target_link_libraries(serializer PRIVATE parser)

file(GLOB_RECURSE MAIN_SRC src/main.cu include/common.cuh)
add_executable(main ${MAIN_SRC})
set_target_properties(main PROPERTIES CUDA_SEPARABLE_COMPILATION ON)

find_package(CUDAToolkit REQUIRED)
target_include_directories(main PRIVATE ${CMAKE_CUDA_TOOLKIT_INCLUDE_DIRECTORIES})
target_link_libraries(main PRIVATE
        CUDA::cudart
        parser
        serializer
)

FetchContent_Declare(
        pugixml
        GIT_REPOSITORY https://github.com/zeux/pugixml.git
        GIT_TAG v1.15
)
FetchContent_MakeAvailable(pugixml)
target_link_libraries(parser PRIVATE pugixml)

# tests
FetchContent_Declare(
        googletest
        GIT_REPOSITORY https://github.com/google/googletest.git
        GIT_TAG v1.17.0
)
FetchContent_MakeAvailable(googletest)

enable_testing()
include(GoogleTest)

file(GLOB_RECURSE PARSER_TESTS_SRC tests/parser/*.cpp)
add_executable(parser_tests ${PARSER_TESTS_SRC})
target_link_libraries(parser_tests PRIVATE parser GTest::gtest_main)
target_compile_definitions(parser_tests PRIVATE DATA_DIR="${CMAKE_CURRENT_SOURCE_DIR}/../data")
gtest_discover_tests(parser_tests PROPERTIES LABELS parser)

file(GLOB_RECURSE SERIALIZER_TESTS_SRC tests/serializer/*.cpp)
add_executable(serializer_tests ${SERIALIZER_TESTS_SRC})
target_link_libraries(serializer_tests PRIVATE serializer GTest::gtest_main)
gtest_discover_tests(serializer_tests PROPERTIES LABELS serializer)