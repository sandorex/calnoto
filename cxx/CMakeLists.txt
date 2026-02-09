cmake_minimum_required(VERSION 3.20)

project(qt-example VERSION 0.1 LANGUAGES CXX)
#
# set(CMAKE_INCLUDE_CURRENT_DIR ON)

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 COMPONENTS Core Quick REQUIRED)

# TODO automatically use debug/release depending on cmake
set(RUST_LIB_NAME ${CMAKE_STATIC_LIBRARY_PREFIX}calnoto_core${CMAKE_STATIC_LIBRARY_SUFFIX})
set(RUST_LIB_PATH ${CMAKE_SOURCE_DIR}/../target/debug/${RUST_LIB_NAME})

# TODO add target alias for this
add_custom_command(
    OUTPUT ${RUST_LIB_PATH}
    COMMAND cargo build
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    DEPENDS ${CMAKE_SOURCE_DIR}/../rust/src/lib.rs
    USES_TERMINAL
)

set(PROJECT_SOURCES
    main.cpp
    qml.qrc
)

qt_add_executable(qt-example
    MANUAL_FINALIZATION
    ${PROJECT_SOURCES}
)
# Define target properties for Android with Qt 6 as:
#    set_property(TARGET qt-example APPEND PROPERTY QT_ANDROID_PACKAGE_SOURCE_DIR
#                 ${CMAKE_CURRENT_SOURCE_DIR}/android)
# For more information, see https://doc.qt.io/qt-6/qt-add-executable.html#target-creation

target_include_directories(qt-example PRIVATE ${CMAKE_SOURCE_DIR}/../rust/target/bindings/)

target_compile_definitions(qt-example
    PRIVATE $<$<OR:$<CONFIG:Debug>,$<CONFIG:RelWithDebInfo>>:QT_QML_DEBUG>)
target_link_libraries(qt-example
    PUBLIC
    Qt${QT_VERSION_MAJOR}::Core
    Qt${QT_VERSION_MAJOR}::Quick
    ${RUST_LIB_PATH}
    # ${CMAKE_SOURCE_DIR}/../target/debug/libcalnoto_core.a
)

# set_target_properties(qt-example PROPERTIES
#     MACOSX_BUNDLE_GUI_IDENTIFIER my.example.com
#     MACOSX_BUNDLE_BUNDLE_VERSION ${PROJECT_VERSION}
#     MACOSX_BUNDLE_SHORT_VERSION_STRING ${PROJECT_VERSION_MAJOR}.${PROJECT_VERSION_MINOR}
# )

qt_import_qml_plugins(qt-example)
qt_finalize_executable(qt-example)

# install(TARGETS qt-example
#     EXPORT qt-example
#     LIBRARY DESTINATION lib
#     ARCHIVE DESTINATION lib
#     RUNTIME DESTINATION bin
#     INCLUDES DESTINATION include
# )
