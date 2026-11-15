
#include "PluginEditor.h"

NGoogleAudioProcessorEditor::NGoogleAudioProcessorEditor(
    NGoogleAudioProcessor& processor)
    : AudioProcessorEditor(&processor),
      audioProcessor(processor)
{
    setSize(1100, 700);
    setResizable(true, true);

    // Browser navigation buttons
    addAndMakeVisible(backButton);
    addAndMakeVisible(forwardButton);
    addAndMakeVisible(reloadButton);
    addAndMakeVisible(homeButton);
    addAndMakeVisible(addressBar);

    backButton.onClick = [this]
    {
        if (browser)
            browser->goBack();
    };

    forwardButton.onClick = [this]
    {
        if (browser)
            browser->goForward();
    };

    reloadButton.onClick = [this]
    {
        if (browser)
            browser->refresh();
    };

    homeButton.onClick = [this]
    {
        if (browser)
            browser->goToURL("https://www.google.com/");
    };

    addressBar.setText("https://www.google.com/");
    addressBar.setReturnKeyStartsNewLine(false);

    addressBar.onReturnKey = [this]
    {
        navigateToAddress();
    };

    // WebView2 profile directory
    auto profileDirectory =
        juce::File::getSpecialLocation(
            juce::File::tempDirectory
        ).getChildFile("N-Google-WebView2");

    auto directoryResult = profileDirectory.createDirectory();

    // Explicit Chromium WebView2 backend
    auto webViewOptions =
        juce::WebBrowserComponent::Options{}
            .withBackend(
                juce::WebBrowserComponent::Options::Backend::webview2
            )
            .withWinWebView2Options(
                juce::WebBrowserComponent::Options::WinWebView2{}
                    .withUserDataFolder(profileDirectory)
                    .withStatusBarDisabled()
                    .withBackgroundColour(
                        juce::Colours::white
                    )
            );

    // Create browser
    browser = std::make_unique<juce::WebBrowserComponent>(
        webViewOptions
    );

    addAndMakeVisible(*browser);

    browser->goToURL("https://www.google.com/");

    addressBar.setColour(
        juce::TextEditor::backgroundColourId,
        juce::Colours::white
    );

    addressBar.setColour(
        juce::TextEditor::textColourId,
        juce::Colours::black
    );

    addressBar.setColour(
        juce::TextEditor::outlineColourId,
        juce::Colour(0xffdadce0)
    );
}

NGoogleAudioProcessorEditor::~NGoogleAudioProcessorEditor()
{
    browser.reset();
}

void NGoogleAudioProcessorEditor::navigateToAddress()
{
    if (!browser)
        return;

    juce::String address = addressBar.getText().trim();

    if (address.isEmpty())
        return;

    // URL or Google search
    if (address.contains(" ") ||
        (!address.contains("://") &&
         !address.contains(".") &&
         address != "localhost"))
    {
        address = "https://www.google.com/search?q="
                + juce::URL::addEscapeChars(address, true);
    }
    else if (!address.startsWithIgnoreCase("http://") &&
             !address.startsWithIgnoreCase("https://"))
    {
        address = "https://" + address;
    }

    addressBar.setText(address, false);
    browser->goToURL(address);
}

void NGoogleAudioProcessorEditor::paint(juce::Graphics& g)
{
    g.fillAll(juce::Colours::white);

    // Toolbar background
    g.setColour(juce::Colour(0xfff8f9fa));
    g.fillRect(0, 0, getWidth(), 55);

    // Bottom toolbar border
    g.setColour(juce::Colour(0xffdadce0));
    g.drawHorizontalLine(54, 0.0f, (float)getWidth());

    // Branding
    g.setColour(juce::Colour(0xff4285f4));
    g.setFont(juce::Font(15.0f, juce::Font::bold));
    g.drawText(
        "N-Google",
        12, 55, 110, 25,
        juce::Justification::centredLeft
    );

    g.setColour(juce::Colours::grey);
    g.setFont(juce::Font(10.0f));
    g.drawText(
        "TG: @vst_ntune",
        12, getHeight() - 20, 150, 15,
        juce::Justification::centredLeft
    );
}

void NGoogleAudioProcessorEditor::resized()
{
    auto area = getLocalBounds();

    auto toolbar = area.removeFromTop(55);

    backButton.setBounds(toolbar.removeFromLeft(42).reduced(4));
    forwardButton.setBounds(toolbar.removeFromLeft(42).reduced(4));
    reloadButton.setBounds(toolbar.removeFromLeft(42).reduced(4));
    homeButton.setBounds(toolbar.removeFromLeft(42).reduced(4));

    addressBar.setBounds(toolbar.reduced(8, 10));

    area.removeFromBottom(20);

    if (browser)
        browser->setBounds(area);
}
