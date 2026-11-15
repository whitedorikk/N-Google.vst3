
#pragma once

#include <JuceHeader.h>
#include "PluginProcessor.h"

class NGoogleAudioProcessorEditor
    : public juce::AudioProcessorEditor
{
public:
    explicit NGoogleAudioProcessorEditor(
        NGoogleAudioProcessor& processor);

    ~NGoogleAudioProcessorEditor() override = default;

    void paint(juce::Graphics& g) override;
    void resized() override;

private:
    static juce::WebBrowserComponent::Options
        createBrowserOptions();

    void navigateToAddress();
    void goHome();

    NGoogleAudioProcessor& audioProcessor;

    juce::WebBrowserComponent browser;

    juce::TextButton backButton { "<" };
    juce::TextButton forwardButton { ">" };
    juce::TextButton reloadButton { "↻" };
    juce::TextButton homeButton { "⌂" };

    juce::TextEditor addressBar;
    juce::Label metadataLabel;

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(
        NGoogleAudioProcessorEditor
    )
};
