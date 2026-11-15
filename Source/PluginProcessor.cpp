
#include "PluginProcessor.h"
#include "PluginEditor.h"

NGoogleAudioProcessor::NGoogleAudioProcessor()
    : AudioProcessor(
        BusesProperties()
            .withInput("Input", juce::AudioChannelSet::stereo(), true)
            .withOutput("Output", juce::AudioChannelSet::stereo(), true)
    )
{
}

void NGoogleAudioProcessor::prepareToPlay(
    double sampleRate,
    int samplesPerBlock)
{
    juce::ignoreUnused(sampleRate, samplesPerBlock);
}

void NGoogleAudioProcessor::releaseResources()
{
}

bool NGoogleAudioProcessor::isBusesLayoutSupported(
    const BusesLayout& layouts) const
{
    const auto& input = layouts.getMainInputChannelSet();
    const auto& output = layouts.getMainOutputChannelSet();

    return input == output
        && !input.isDisabled()
        && (input == juce::AudioChannelSet::mono()
            || input == juce::AudioChannelSet::stereo());
}

void NGoogleAudioProcessor::processBlock(
    juce::AudioBuffer<float>& buffer,
    juce::MidiBuffer& midiMessages)
{
    juce::ignoreUnused(midiMessages);

    // Audio passthrough: browser does not modify audio.
    juce::ScopedNoDenormals noDenormals;

    juce::ignoreUnused(buffer);
}

juce::AudioProcessorEditor*
NGoogleAudioProcessor::createEditor()
{
    return new NGoogleAudioProcessorEditor(*this);
}

bool NGoogleAudioProcessor::hasEditor() const
{
    return true;
}

const juce::String NGoogleAudioProcessor::getName() const
{
    return JucePlugin_Name;
}

bool NGoogleAudioProcessor::acceptsMidi() const
{
    return false;
}

bool NGoogleAudioProcessor::producesMidi() const
{
    return false;
}

bool NGoogleAudioProcessor::isMidiEffect() const
{
    return false;
}

double NGoogleAudioProcessor::getTailLengthSeconds() const
{
    return 0.0;
}

int NGoogleAudioProcessor::getNumPrograms()
{
    return 1;
}

int NGoogleAudioProcessor::getCurrentProgram()
{
    return 0;
}

void NGoogleAudioProcessor::setCurrentProgram(int index)
{
    juce::ignoreUnused(index);
}

const juce::String NGoogleAudioProcessor::getProgramName(int index)
{
    juce::ignoreUnused(index);
    return {};
}

void NGoogleAudioProcessor::changeProgramName(
    int index,
    const juce::String& newName)
{
    juce::ignoreUnused(index, newName);
}

void NGoogleAudioProcessor::getStateInformation(
    juce::MemoryBlock& destData)
{
    juce::ignoreUnused(destData);
}

void NGoogleAudioProcessor::setStateInformation(
    const void* data,
    int sizeInBytes)
{
    juce::ignoreUnused(data, sizeInBytes);
}

juce::AudioProcessor* JUCE_CALLTYPE createPluginFilter()
{
    return new NGoogleAudioProcessor();
}
