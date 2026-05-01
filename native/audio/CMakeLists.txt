cmake_minimum_required(VERSION 3.27)
project(pdj_audio VERSION 0.0.1 LANGUAGES CXX)

# Use C++20 everywhere.
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Strict warnings — keep the codebase clean from the start.
if(MSVC)
  add_compile_options(/W4 /WX)
else()
  add_compile_options(-Wall -Wextra -Wpedantic -Werror)
endif()

# ---------------------------------------------------------------------------
# Find external libraries via pkg-config
# ---------------------------------------------------------------------------
find_package(PkgConfig REQUIRED)
pkg_check_modules(PORTAUDIO REQUIRED IMPORTED_TARGET portaudio-2.0)
pkg_check_modules(SNDFILE   REQUIRED IMPORTED_TARGET sndfile)

# Threads (pthreads on Linux/macOS).
set(CMAKE_THREAD_PREFER_PTHREAD ON)
set(THREADS_PREFER_PTHREAD_FLAG ON)
find_package(Threads REQUIRED)

# ---------------------------------------------------------------------------
# pdj_audio shared library
# ---------------------------------------------------------------------------
# Compiled as a shared library that pdj-engine-bridge links against via FFI.

add_library(pdj_audio SHARED
    src/engine.cpp
    src/decoder.cpp
    src/deck.cpp
    src/mixer.cpp
    src/portaudio_backend.cpp
    src/bpm.cpp
)

target_include_directories(pdj_audio
    PUBLIC  ${CMAKE_CURRENT_SOURCE_DIR}/include
    PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/src
)

target_link_libraries(pdj_audio
    PRIVATE PkgConfig::PORTAUDIO
    PRIVATE PkgConfig::SNDFILE
    PRIVATE Threads::Threads
)

# Position-independent code so the shared lib can be relocated.
set_property(TARGET pdj_audio PROPERTY POSITION_INDEPENDENT_CODE ON)

# ---------------------------------------------------------------------------
# GoogleTest for C++ unit tests
# ---------------------------------------------------------------------------
include(FetchContent)

# Allow disabling internet fetch — handy in CI environments without network.
option(PDJ_FETCH_GTEST "Download GoogleTest at configure time" ON)

if(PDJ_FETCH_GTEST)
  FetchContent_Declare(
    googletest
    URL https://github.com/google/googletest/archive/refs/tags/v1.14.0.zip
    DOWNLOAD_EXTRACT_TIMESTAMP TRUE
  )
  set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
  FetchContent_MakeAvailable(googletest)

  enable_testing()
  include(GoogleTest)

  add_executable(pdj_audio_tests
      tests/engine_stub_test.cpp
      tests/ringbuffer_test.cpp
      tests/bpm_test.cpp
  )

  target_include_directories(pdj_audio_tests PRIVATE
      ${CMAKE_CURRENT_SOURCE_DIR}/src
  )

  target_link_libraries(pdj_audio_tests
      PRIVATE pdj_audio
      PRIVATE GTest::gtest_main
  )

  gtest_discover_tests(pdj_audio_tests)
endif()
