cmake_minimum_required(VERSION 3.10)

project(c-verify VERSION 1.0 LANGUAGES C)

get_filename_component(REAL_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)
set(SEARCH_PATH "${REAL_ROOT}/rust/pact_ffi/build/install/lib/cmake")

find_package(PactFfi REQUIRED CONFIG PATHS ${SEARCH_PATH})

add_executable(c-verify src/main.c)

target_link_libraries(c-verify PRIVATE PactFfi pthread dl m)
