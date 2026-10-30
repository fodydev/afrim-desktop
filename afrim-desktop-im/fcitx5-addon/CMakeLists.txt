cmake_minimum_required(VERSION 3.16)

project(fcitx5-afrim-desktop VERSION 0.1.0)


set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# fcitx5 only loads addon libraries from its own addon dir (e.g.
# /usr/lib/x86_64-linux-gnu/fcitx5), so CMake's /usr/local default gives
# "Could not locate library afrim-desktop.so". Default to /usr instead.
if(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
    set(CMAKE_INSTALL_PREFIX "/usr" CACHE PATH "Install prefix" FORCE)
endif()

# Find dependencies
find_package(ECM REQUIRED)
set(CMAKE_MODULE_PATH ${ECM_MODULE_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/cmake)

find_package(Fcitx5Core 5.0 REQUIRED)
find_package(Fcitx5Module COMPONENTS Notifications QuickPhrase)
find_package(PkgConfig REQUIRED)
pkg_check_modules(XKBCommon REQUIRED xkbcommon)

include(FeatureSummary)
include(GNUInstallDirs)
include(ECMSetupVersion)
include(ECMUninstallTarget)

# Use fcitx5 compiler settings
include("${FCITX_INSTALL_CMAKECONFIG_DIR}/Fcitx5Utils/Fcitx5CompilerSettings.cmake")

# Path to the Rust library
set(AFRIM_DESKTOP_RUST_LIB "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release/libafrim_desktop_im.so")
set(AFRIM_DESKTOP_INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../include")

# Build Rust library as a custom target
find_program(CARGO cargo REQUIRED)
add_custom_target(afrim_desktop_rust_lib ALL
    COMMAND ${CARGO} rustc --release -p afrim-desktop-im --crate-type cdylib
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/../.."
    COMMENT "Building afrim-desktop-im Rust library"
    BYPRODUCTS "${AFRIM_DESKTOP_RUST_LIB}"
)

# Add the addon
add_library(afrim-desktop MODULE
    src/afrim-desktop.cpp
)
add_dependencies(afrim-desktop afrim_desktop_rust_lib)

target_include_directories(afrim-desktop PRIVATE
    ${AFRIM_DESKTOP_INCLUDE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${XKBCommon_INCLUDE_DIRS}
)

target_link_libraries(afrim-desktop
    Fcitx5::Core
    Fcitx5::Config
    ${AFRIM_DESKTOP_RUST_LIB}
    ${XKBCommon_LIBRARIES}
)

# Remove lib prefix (fcitx5 expects "afrim-desktop.so" not "libafrim-desktop.so")
set_target_properties(afrim-desktop PROPERTIES PREFIX "")

# Set RPATH so fcitx5 can find libafrim_desktop_im.so at runtime
# $ORIGIN means "same directory as this library"
set_target_properties(afrim-desktop PROPERTIES
    BUILD_RPATH "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release"
    INSTALL_RPATH "\$ORIGIN"
    BUILD_WITH_INSTALL_RPATH FALSE
)

# Install the addon
install(TARGETS afrim-desktop DESTINATION "${FCITX_INSTALL_ADDONDIR}")

# Install Rust library alongside the addon
install(FILES "${AFRIM_DESKTOP_RUST_LIB}" DESTINATION "${FCITX_INSTALL_ADDONDIR}")

# Install addon config
install(FILES afrim-desktop-addon.conf.in DESTINATION "${FCITX_INSTALL_PKGDATADIR}/addon"
    RENAME afrim-desktop.conf)

# Install input method config
install(FILES afrim-desktop.conf DESTINATION "${FCITX_INSTALL_PKGDATADIR}/inputmethod")

# Install metainfo
install(FILES org.fcitx.Fcitx5.Addon.AfrimDesktop.metainfo.xml.in
    DESTINATION "${CMAKE_INSTALL_DATAROOTDIR}/metainfo"
    RENAME org.fcitx.Fcitx5.Addon.AfrimDesktop.metainfo.xml)

# Install icons at multiple sizes for XDG icon theme
install(FILES icons/fcitx-afrim-desktop-16.png
    DESTINATION "${CMAKE_INSTALL_DATAROOTDIR}/icons/hicolor/16x16/apps/"
    RENAME fcitx-afrim-desktop.png)
install(FILES icons/fcitx-afrim-desktop-24.png
    DESTINATION "${CMAKE_INSTALL_DATAROOTDIR}/icons/hicolor/24x24/apps/"
    RENAME fcitx-afrim-desktop.png)
install(FILES icons/fcitx-afrim-desktop-32.png
    DESTINATION "${CMAKE_INSTALL_DATAROOTDIR}/icons/hicolor/32x32/apps/"
    RENAME fcitx-afrim-desktop.png)
install(FILES icons/fcitx-afrim-desktop-48.png
    DESTINATION "${CMAKE_INSTALL_DATAROOTDIR}/icons/hicolor/48x48/apps/"
    RENAME fcitx-afrim-desktop.png)
install(FILES icons/fcitx-afrim-desktop-128.png
    DESTINATION "${CMAKE_INSTALL_DATAROOTDIR}/icons/hicolor/128x128/apps/"
    RENAME fcitx-afrim-desktop.png)

feature_summary(WHAT ALL FATAL_ON_MISSING_REQUIRED_PACKAGES)
