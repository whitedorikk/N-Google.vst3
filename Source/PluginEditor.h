
#pragma once

#include <JuceHeader.h>
#include "PluginProcessor.h"

class NGoogleAudioProcessorEditor : public juce::AudioProcessorEditor
{
public:
    explicit NGoogleAudioProcessorEditor(NGoogleAudioProcessor&);
    ~NGoogleAudioProcessorEditor() override;

    void paint(juce::Graphics&) override;
    void resized() override;

private:
    NGoogleAudioProcessor& audioProcessor;

    std::unique_ptr<juce::WebBrowserComponent> browser;

    juce::TextButton backButton { "<" };
    juce::TextButton forwardButton { ">" };
    juce::TextButton reloadButton { "↻" };
    juce::TextButton homeButton { "⌂" };

    juce::TextEditor addressBar;

    void navigateToAddress();

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(
        NGoogleAudioProcessorEditor
    )
};
