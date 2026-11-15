
#include "PluginEditor.h"

juce::WebBrowserComponent::Options
NGoogleAudioProcessorEditor::createBrowserOptions()
{
    auto profileFolder =
        juce::File::getSpecialLocation(
            juce::File::userApplicationDataDirectory
        ).getChildFile("N-Google")
         .getChildFile("WebView2");

    profileFolder.createDirectory();

    const auto webView2Options =
        juce::WebBrowserComponent::Options::WinWebView2{}
            .withUserDataFolder(profileFolder)
            .withStatusBarDisabled();

    return juce::WebBrowserComponent::Options{}
        .withBackend(
            juce::WebBrowserComponent::Options::Backend::webview2
        )
        .withWinWebView2Options(webView2Options)
        .withKeepPageLoadedWhenBrowserIsHidden();
}

NGoogleAudioProcessorEditor::NGoogleAudioProcessorEditor(
    NGoogleAudioProcessor& processor)
    : AudioProcessorEditor(&processor),
      audioProcessor(processor),
      browser(createBrowserOptions())
{
    setSize(1100, 720);
    setResizable(true, true);
    setResizeLimits(640, 420, 2400, 1600);

    addAndMakeVisible(browser);

    addAndMakeVisible(backButton);
    addAndMakeVisible(forwardButton);
    addAndMakeVisible(reloadButton);
    addAndMakeVisible(homeButton);
    addAndMakeVisible(addressBar);
    addAndMakeVisible(metadataLabel);

    backButton.onClick = [this]
    {
        browser.goBack();
    };

    forwardButton.onClick = [this]
    {
        browser.goForward();
    };

    reloadButton.onClick = [this]
    {
        browser.refresh();
    };

    homeButton.onClick = [this]
    {
        goHome();
    };

    addressBar.setText("https://www.google.com");
    addressBar.setFont(juce::Font(15.0f));
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
    addressBar.setBorder(
        juce::BorderSize<int>(8, 12, 8, 12)
    );

    addressBar.onReturnKey = [this]
    {
        navigateToAddress();
    };

    metadataLabel.setText(
        "TG: @vst_ntune",
        juce::dontSendNotification
    );
    metadataLabel.setFont(juce::Font(11.0f));
    metadataLabel.setColour(
        juce::Label::textColourId,
        juce::Colour(0xff5f6368)
    );
    metadataLabel.setJustificationType(
        juce::Justification::centred
    );

    browser.goToURL("https://www.google.com");
}

void NGoogleAudioProcessorEditor::paint(
    juce::Graphics& g)
{
    g.fillAll(juce::Colour(0xfff8f9fa));

    g.setColour(juce::Colour(0xffdadce0));
    g.drawLine(
        0.0f,
        48.0f,
        static_cast<float>(getWidth()),
        48.0f,
        1.0f
    );
}

void NGoogleAudioProcessorEditor::resized()
{
    auto area = getLocalBounds();

    auto toolbar = area.removeFromTop(48);

    backButton.setBounds(
        toolbar.removeFromLeft(42).reduced(4)
    );

    forwardButton.setBounds(
        toolbar.removeFromLeft(42).reduced(4)
    );

    reloadButton.setBounds(
        toolbar.removeFromLeft(42).reduced(4)
    );

    homeButton.setBounds(
        toolbar.removeFromLeft(42).reduced(4)
    );

    metadataLabel.setBounds(
        toolbar.removeFromRight(112).reduced(2)
    );

    addressBar.setBounds(
        toolbar.reduced(5, 7)
    );

    browser.setBounds(area);
}

void NGoogleAudioProcessorEditor::navigateToAddress()
{
    auto input = addressBar.getText().trim();

    if (input.isEmpty())
        return;

    if (input.startsWithIgnoreCase("https://")
        || input.startsWithIgnoreCase("http://"))
    {
        browser.goToURL(input);
        return;
    }

    if (!input.contains(" ")
        && input.contains("."))
    {
        browser.goToURL("https://" + input);
        return;
    }

    auto searchUrl =
        juce::URL("https://www.google.com/search")
            .withParameter("q", input);

    browser.goToURL(searchUrl.toString(true));
}

void NGoogleAudioProcessorEditor::goHome()
{
    addressBar.setText(
        "https://www.google.com",
        juce::dontSendNotification
    );

    browser.goToURL("https://www.google.com");
}
