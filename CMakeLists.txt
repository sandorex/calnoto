cmake_minimum_required(VERSION 3.20)

project(qt-example VERSION 0.1 LANGUAGES CXX)

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 COMPONENTS Core Quick REQUIRED)

# TODO split debug and release builds

# path must be manually provided (set in 'cargo xtask')
if (NOT ((DEFINED RUST_LIB) AND (DEFINED RUST_INCLUDE)))
    message(FATAL_ERROR "Please provide -DRUST_LIB and -DRUST_INCLUDE")
endif()

add_library(rust STATIC IMPORTED)
set_target_properties(rust
    PROPERTIES
        IMPORTED_LOCATION               "${RUST_LIB}"
        INTERFACE_INCLUDE_DIRECTORIES   "${RUST_INCLUDE}"
)

qt_add_executable(qt-example
    MANUAL_FINALIZATION
    ${CMAKE_SOURCE_DIR}/src/main.cpp
    ${CMAKE_SOURCE_DIR}/src/qml.qrc
)
# Define target properties for Android with Qt 6 as:
#    set_property(TARGET qt-example APPEND PROPERTY QT_ANDROID_PACKAGE_SOURCE_DIR
#                 ${CMAKE_CURRENT_SOURCE_DIR}/android)
# For more information, see https://doc.qt.io/qt-6/qt-add-executable.html#target-creation

target_compile_definitions(qt-example
    PRIVATE $<$<OR:$<CONFIG:Debug>,$<CONFIG:RelWithDebInfo>>:QT_QML_DEBUG>)
target_link_libraries(qt-example
    PUBLIC
    Qt6::Core
    Qt6::Quick
    rust
)

# set_target_properties(qt-example PROPERTIES
#     MACOSX_BUNDLE_GUI_IDENTIFIER my.example.com
#     MACOSX_BUNDLE_BUNDLE_VERSION ${PROJECT_VERSION}
#     MACOSX_BUNDLE_SHORT_VERSION_STRING ${PROJECT_VERSION_MAJOR}.${PROJECT_VERSION_MINOR}
# )

qt_import_qml_plugins(qt-example)
qt_finalize_executable(qt-example)

# TODO installation
# install(TARGETS qt-example
#     EXPORT qt-example
#     LIBRARY DESTINATION lib
#     ARCHIVE DESTINATION lib
#     RUNTIME DESTINATION bin
#     INCLUDES DESTINATION include
# )
