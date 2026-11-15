
cmake_minimum_required(VERSION 3.22)

project(NGoogle VERSION 1.0.0)

# JUCE
add_subdirectory(JUCE)

# Plugin
juce_add_plugin(NGoogle
    COMPANY_NAME "vst_ntune"
    BUNDLE_ID "com.vst_ntune.ngoogle"
    PLUGIN_MANUFACTURER_CODE Vstn
    PLUGIN_CODE NGgl

    PRODUCT_NAME "N-Google"
    DESCRIPTION "Google Browser VST3 Plugin"

    FORMATS VST3

    IS_SYNTH FALSE
    IS_MIDI_EFFECT FALSE
    NEEDS_MIDI_INPUT FALSE
    NEEDS_MIDI_OUTPUT FALSE
    IS_MIDI_EFFECT FALSE

    NEEDS_WEBVIEW2 TRUE

    EDITOR_WANTS_KEYBOARD_FOCUS TRUE

    COPY_PLUGIN_AFTER_BUILD FALSE
)

# Generate JuceHeader.h
juce_generate_juce_header(NGoogle)

# Source files
target_sources(NGoogle
    PRIVATE
        Source/PluginProcessor.cpp
        Source/PluginProcessor.h
        Source/PluginEditor.cpp
        Source/PluginEditor.h
)

# C++ standard
target_compile_features(NGoogle PRIVATE cxx_std_17)

# JUCE modules
target_link_libraries(NGoogle
    PRIVATE
        juce::juce_audio_utils
        juce::juce_audio_processors
        juce::juce_gui_extra
        juce::juce_gui_basics
        juce::juce_core
    PUBLIC
        juce::juce_recommended_config_flags
        juce::juce_recommended_lto_flags
        juce::juce_recommended_warning_flags
)

# Browser configuration
target_compile_definitions(NGoogle
    PUBLIC
        JUCE_WEB_BROWSER=1
        JUCE_USE_WIN_WEBVIEW2_WITH_STATIC_LINKING=1
        JUCE_VST3_CAN_REPLACE_VST2=0
        JUCE_DISPLAY_SPLASH_SCREEN=0
        JUCE_REPORT_APP_USAGE=0
)

# Windows
if(WIN32)
    target_link_libraries(NGoogle PRIVATE
        ole32
        oleaut32
        uuid
        version
        shlwapi
    )
endif()
