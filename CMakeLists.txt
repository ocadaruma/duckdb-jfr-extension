# temporary commented out for dynamic linking

#cmake_minimum_required(VERSION 3.22)
#set(TARGET_NAME jfr)
#set(EXTENSION_NAME ${TARGET_NAME}_extension)
#project(${TARGET_NAME})
#
#include(FetchContent)
#
#FetchContent_Declare(
#    Corrosion
#    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
#    GIT_TAG v0.4.2
#)
#FetchContent_MakeAvailable(Corrosion)
#
#corrosion_import_crate(MANIFEST_PATH ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml)
#
#set(ALL_SOURCES src/bridge.hpp src/bridge.cpp)
#
#build_loadable_extension(${TARGET_NAME} ${ALL_SOURCES})
#
#set(LIB_NAME ${TARGET_NAME}_loadable_extension)
#
#set_target_properties(${LIB_NAME} PROPERTIES LINKER_LANGUAGE CXX)
#target_link_libraries(${LIB_NAME} "${CMAKE_CURRENT_BINARY_DIR}/lib${EXTENSION_NAME}.a")
