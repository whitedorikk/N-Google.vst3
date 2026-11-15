#include "PluginEditor.h"

NGoogleAudioProcessorEditor::NGoogleAudioProcessorEditor(NGoogleAudioProcessor& p)
    : AudioProcessorEditor(&p)
{
    setSize(1000, 680);
    setResizable(true, true);

    addAndMakeVisible(backButton);
    addAndMakeVisible(forwardButton);
    addAndMakeVisible(reloadButton);
    addAndMakeVisible(homeButton);
    addAndMakeVisible(addressBar);
    addAndMakeVisible(goButton);
    addAndMakeVisible(browser);

    addressBar.setText("https://www.google.com", juce::dontSendNotification);
    addressBar.setReturnKeyStartsNewLine(false);
    addressBar.onReturnKey = [this] { navigate(); };
    goButton.onClick = [this] { navigate(); };
    backButton.onClick = [this] { browser.goBack(); };
    forwardButton.onClick = [this] { browser.goForward(); };
    reloadButton.onClick = [this] { browser.refresh(); };
    homeButton.onClick = [this]
    {
        addressBar.setText("https://www.google.com", juce::dontSendNotification);
        navigate();
    };

    browser.goToURL("https://www.google.com");
}

void NGoogleAudioProcessorEditor::paint(juce::Graphics& g)
{
    g.fillAll(juce::Colour(0xfff8f9fa));
    g.setColour(juce::Colour(0xffdadce0));
    g.drawLine(0.0f, 42.0f, static_cast<float>(getWidth()), 42.0f, 1.0f);
}

void NGoogleAudioProcessorEditor::resized()
{
    auto area = getLocalBounds();
    auto toolbar = area.removeFromTop(42).reduced(5, 4);

    backButton.setBounds(toolbar.removeFromLeft(34));
    toolbar.removeFromLeft(3);
    forwardButton.setBounds(toolbar.removeFromLeft(34));
    toolbar.removeFromLeft(3);
    reloadButton.setBounds(toolbar.removeFromLeft(34));
    toolbar.removeFromLeft(3);
    homeButton.setBounds(toolbar.removeFromLeft(34));
    toolbar.removeFromLeft(8);
    goButton.setBounds(toolbar.removeFromRight(48));
    toolbar.removeFromRight(4);
    addressBar.setBounds(toolbar);
    browser.setBounds(area);
}

void NGoogleAudioProcessorEditor::navigate()
{
    auto target = addressBar.getText().trim();
    if (target.isEmpty())
        return;

    const bool looksLikeUrl = target.containsChar('.') && !target.containsChar(' ');
    if (!target.startsWithIgnoreCase("http://") && !target.startsWithIgnoreCase("https://"))
        target = looksLikeUrl ? "https://" + target
                              : "https://www.google.com/search?q=" + juce::URL::addEscapeChars(target, true);

    addressBar.setText(target, juce::dontSendNotification);
    browser.goToURL(target);
}
