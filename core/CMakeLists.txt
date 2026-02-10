set(RUST_LIB_NAME calnoto_core)
set(RUST_BUILD_DIR ${CMAKE_CURRENT_BINARY_DIR}/target)

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(RUST_LIB ${RUST_BUILD_DIR}/debug/${CMAKE_STATIC_LIBRARY_PREFIX}${RUST_LIB_NAME}${CMAKE_STATIC_LIBRARY_SUFFIX})
    set(RUST_CMD_SUFFIX "")
else()
    set(RUST_LIB ${RUST_BUILD_DIR}/release/${CMAKE_STATIC_LIBRARY_PREFIX}${RUST_LIB_NAME}${CMAKE_STATIC_LIBRARY_SUFFIX})
    set(RUST_CMD_SUFFIX "--release")
endif()

add_custom_command(
    OUTPUT ${RUST_LIB}
    COMMAND cargo build --manifest-path "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml" --target-dir "${RUST_BUILD_DIR}" ${RUST_CMD_SUFFIX}
    USES_TERMINAL
    VERBATIM
)

add_custom_target(calnoto-core-build DEPENDS "${RUST_LIB}")

add_library(calnoto-core STATIC IMPORTED GLOBAL)
add_library(calnoto::core ALIAS calnoto-core)
set_target_properties(calnoto-core
    PROPERTIES
        IMPORTED_LOCATION               "${RUST_LIB}"
        INTERFACE_INCLUDE_DIRECTORIES   "${CMAKE_CURRENT_SOURCE_DIR}/include"
)
add_dependencies(calnoto-core calnoto-core-build)
