cmake_minimum_required(VERSION 3.27)
project(pdj_audio VERSION 0.0.1 LANGUAGES CXX)

# Use C++20 everywhere.
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Strict warnings — treat them as errors so the project stays clean.
if(MSVC)
  add_compile_options(/W4 /WX)
else()
  add_compile_options(-Wall -Wextra -Wpedantic -Werror)
endif()

# ---------------------------------------------------------------------------
# pdj_audio shared library
# ---------------------------------------------------------------------------
# The audio engine is compiled as a shared library and loaded by pdj-engine-
# bridge via Rust's FFI.  A C ABI header (include/pdj_engine.h) is the
# contract between Rust and C++.

add_library(pdj_audio SHARED
    src/engine.cpp
)

target_include_directories(pdj_audio
    PUBLIC  ${CMAKE_CURRENT_SOURCE_DIR}/include
    PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/src
)

# ---------------------------------------------------------------------------
# GoogleTest for C++ unit tests
# ---------------------------------------------------------------------------
include(FetchContent)
FetchContent_Declare(
  googletest
  URL https://github.com/google/googletest/archive/refs/tags/v1.14.0.zip
  URL_HASH SHA256=1fb1bb23301945ef3d87c9264e56e1e5c50b8a17d8f71f0e68f97b6c9a9b28b6
  DOWNLOAD_EXTRACT_TIMESTAMP TRUE
)
# Prevent GoogleTest from overriding compiler / linker settings on Windows.
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(googletest)

enable_testing()
include(GoogleTest)

add_executable(pdj_audio_tests
    tests/engine_stub_test.cpp
)
target_link_libraries(pdj_audio_tests
    PRIVATE pdj_audio
    PRIVATE GTest::gtest_main
)
gtest_discover_tests(pdj_audio_tests)
