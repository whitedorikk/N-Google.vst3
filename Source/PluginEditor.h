#pragma once
#include <JuceHeader.h>
#include "PluginProcessor.h"

class NGoogleAudioProcessorEditor final : public juce::AudioProcessorEditor
{
public:
    explicit NGoogleAudioProcessorEditor(NGoogleAudioProcessor&);
    ~NGoogleAudioProcessorEditor() override = default;

    void paint(juce::Graphics&) override;
    void resized() override;

private:
    void navigate();
    void setAddressFromCurrentPage();

    juce::WebBrowserComponent browser;
    juce::TextButton backButton { "←" };
    juce::TextButton forwardButton { "→" };
    juce::TextButton reloadButton { "↻" };
    juce::TextButton homeButton { "G" };
    juce::TextEditor addressBar;
    juce::TextButton goButton { "Go" };

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(NGoogleAudioProcessorEditor)
};
