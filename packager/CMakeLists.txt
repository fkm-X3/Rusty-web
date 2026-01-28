cmake_minimum_required(VERSION 3.20)
project(RustyWebLauncher LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(RustyWebLauncher main.cpp)

# Link against the Rust static library produced by Cargo.
# Make sure you've run: cargo build --release
# Adjust the path/name if Cargo outputs a different .lib file.
set(RUST_LIB_PATH "${CMAKE_CURRENT_LIST_DIR}/../target/release/rusty_web_core.lib")

if(NOT EXISTS "${RUST_LIB_PATH}")
    message(WARNING "Rust static library not found at ${RUST_LIB_PATH}. Build Rust first with `cargo build --release`.")
endif()

target_link_libraries(RustyWebLauncher PRIVATE "${RUST_LIB_PATH}")
