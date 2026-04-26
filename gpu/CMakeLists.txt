cmake_minimum_required(VERSION 3.28)
project(gpu-timetabling LANGUAGES CXX CUDA)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CUDA_STANDARD 20)
set(CMAKE_CUDA_STANDARD_REQUIRED ON)
set(CMAKE_CUDA_ARCHITECTURES native)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
include(FetchContent)

include_directories(include)

set(CXX_COMPILE_OPTIONS_DEBUG "-O0;-g;-Wall;-Wextra;-Wpedantic")
set(CXX_COMPILE_OPTIONS_RELEASE "-O3;-march=native;-Wall;-Wextra;-Wpedantic")
set(CUDA_COMPILE_OPTIONS_DEBUG "-O0;-g;-G;-v;-fno-omit-frame-pointer")
set(CUDA_COMPILE_OPTIONS_RELEASE "-O3;--use_fast_math")

file(GLOB KERNELS_SRC src/kernels/*.cu)
add_library(kernels OBJECT ${KERNELS_SRC})

file(GLOB EXECUTOR_SRC src/executor/*.cpp)
add_library(executor OBJECT ${EXECUTOR_SRC})

file(GLOB PARSER_SRC src/parser/*.cpp)
add_library(parser OBJECT ${PARSER_SRC})

file(GLOB SERIALIZER_SRC src/serializer/*.cpp)
add_library(serializer OBJECT ${SERIALIZER_SRC})
target_link_libraries(serializer PRIVATE parser)

add_executable(main src/main.cu)

if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    target_compile_options(kernels PRIVATE ${CUDA_COMPILE_OPTIONS_DEBUG})
    target_compile_options(executor PRIVATE ${CXX_COMPILE_OPTIONS_DEBUG})
    target_compile_options(parser PRIVATE ${CXX_COMPILE_OPTIONS_DEBUG})
    target_compile_options(serializer PRIVATE ${CXX_COMPILE_OPTIONS_DEBUG})
    target_compile_options(main PRIVATE ${CUDA_COMPILE_OPTIONS_DEBUG})
else ()
    target_compile_options(kernels PRIVATE ${CUDA_COMPILE_OPTIONS_RELEASE})
    target_compile_options(executor PRIVATE ${CXX_COMPILE_OPTIONS_RELEASE})
    target_compile_options(parser PRIVATE ${CXX_COMPILE_OPTIONS_RELEASE})
    target_compile_options(serializer PRIVATE ${CXX_COMPILE_OPTIONS_RELEASE})
    target_compile_options(main PRIVATE ${CUDA_COMPILE_OPTIONS_RELEASE})
endif ()

find_package(CUDAToolkit REQUIRED)
target_include_directories(main PRIVATE ${CMAKE_CUDA_TOOLKIT_INCLUDE_DIRECTORIES})
target_link_libraries(main PRIVATE
        executor
        kernels
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

file(GLOB PARSER_TESTS_SRC tests/parser/*.cpp)
add_executable(parser_tests ${PARSER_TESTS_SRC})
target_link_libraries(parser_tests PRIVATE pugixml parser GTest::gtest_main)
target_compile_definitions(parser_tests PRIVATE DATA_DIR="${CMAKE_CURRENT_SOURCE_DIR}/../data")
gtest_discover_tests(parser_tests PROPERTIES LABELS parser)

file(GLOB SERIALIZER_TESTS_SRC tests/serializer/*.cpp)
add_executable(serializer_tests ${SERIALIZER_TESTS_SRC})
target_link_libraries(serializer_tests PRIVATE serializer GTest::gtest_main)
gtest_discover_tests(serializer_tests PROPERTIES LABELS serializer)