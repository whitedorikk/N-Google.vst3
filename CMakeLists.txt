
cmake_minimum_required(VERSION 3.22)

project(NGoogle VERSION 1.0.0)

include(FetchContent)

set(JUCE_WEBVIEW2_PACKAGE_LOCATION
    "${CMAKE_CURRENT_SOURCE_DIR}/third_party/nuget"
    CACHE PATH "WebView2 NuGet package location"
)

FetchContent_Declare(
    JUCE
    GIT_REPOSITORY https://github.com/juce-framework/JUCE.git
    GIT_TAG 8.0.12
    GIT_SHALLOW TRUE
)

FetchContent_MakeAvailable(JUCE)

juce_add_plugin(NGoogle
    COMPANY_NAME "vst_ntune"
    BUNDLE_ID "com.vst_ntune.ngoogle"
    PLUGIN_MANUFACTURER_CODE VstN
    PLUGIN_CODE NGo1

    PRODUCT_NAME "N-Google"
    PLUGIN_NAME "N-Google"
    DESCRIPTION "Google-powered browser plugin"

    FORMATS VST3

    VST3_CATEGORIES Fx
    IS_SYNTH FALSE
    IS_MIDI_EFFECT FALSE

    NEEDS_WEBVIEW2 TRUE
    EDITOR_WANTS_KEYBOARD_FOCUS TRUE
)

juce_generate_juce_header(NGoogle)

target_sources(NGoogle
    PRIVATE
        Source/PluginProcessor.cpp
        Source/PluginProcessor.h
        Source/PluginEditor.cpp
        Source/PluginEditor.h
)

target_compile_features(NGoogle PRIVATE cxx_std_17)

target_compile_definitions(NGoogle
    PUBLIC
        JUCE_WEB_BROWSER=1
        JUCE_USE_WIN_WEBVIEW2_WITH_STATIC_LINKING=1
        JUCE_VST3_CAN_REPLACE_VST2=0
)

target_link_libraries(NGoogle
    PRIVATE
        juce::juce_audio_utils
        juce::juce_gui_extra
        juce::juce_recommended_config_flags
        juce::juce_recommended_lto_flags
        juce::juce_recommended_warning_flags
)

if(WIN32)
    target_link_libraries(NGoogle PRIVATE
        juce::juce_webview2
    )
endif()
