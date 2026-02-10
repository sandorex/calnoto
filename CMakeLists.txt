cmake_minimum_required(VERSION 3.20)

project(calnoto VERSION 0.1.0 LANGUAGES CXX)

# default to release
if(CMAKE_BUILD_TYPE STREQUAL "")
    set(CMAKE_BUILD_TYPE "Release")
endif()

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 COMPONENTS Core Quick REQUIRED)

message("Build type: ${CMAKE_BUILD_TYPE}")

add_subdirectory(core)

set(SOURCES
    src/main.cpp
    src/qml.qrc
)

qt_add_executable(calnoto MANUAL_FINALIZATION ${SOURCES})

# Define target properties for Android with Qt 6 as:
#    set_property(TARGET calnoto APPEND PROPERTY QT_ANDROID_PACKAGE_SOURCE_DIR
#                 ${CMAKE_CURRENT_SOURCE_DIR}/android)
# For more information, see https://doc.qt.io/qt-6/qt-add-executable.html#target-creation

target_compile_definitions(calnoto
    PRIVATE
    $<$<OR:$<CONFIG:Debug>,$<CONFIG:RelWithDebInfo>>:QT_QML_DEBUG>
)

target_link_libraries(calnoto
    PUBLIC
    Qt6::Core
    Qt6::Quick
    calnoto::core
)

set_target_properties(calnoto PROPERTIES
    MACOSX_BUNDLE_GUI_IDENTIFIER io.sandorex.github.calnoto
    MACOSX_BUNDLE_BUNDLE_VERSION ${PROJECT_VERSION}
    MACOSX_BUNDLE_SHORT_VERSION_STRING ${PROJECT_VERSION_MAJOR}.${PROJECT_VERSION_MINOR}
)

qt_import_qml_plugins(calnoto)
qt_finalize_executable(calnoto)

# TODO installation
# install(TARGETS calnoto
#     EXPORT calnoto
#     LIBRARY DESTINATION lib
#     ARCHIVE DESTINATION lib
#     RUNTIME DESTINATION bin
#     INCLUDES DESTINATION include
# )
