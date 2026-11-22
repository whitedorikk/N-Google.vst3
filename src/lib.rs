//! N-Google (TG: @vst_ntune): a VST3 utility plugin with an embedded WebView2 browser. Audio passes through untouched.
use nih_plug::prelude::*;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

#[cfg(windows)]
mod editor;

pub const HOME: &str = "https://www.google.com";

#[derive(Params)]
struct NGoogleParams {
    /// last visited page, saved inside the DAW project
    #[persist = "url"]
    url: Arc<Mutex<String>>,
}

pub struct NGoogle {
    params: Arc<NGoogleParams>,
}

impl Default for NGoogle {
    fn default() -> Self {
        Self {
            params: Arc::new(NGoogleParams {
                url: Arc::new(Mutex::new(HOME.to_string())),
            }),
        }
    }
}

impl Plugin for NGoogle {
    const NAME: &'static str = "N-Google";
    const VENDOR: &'static str = "TG: @vst_ntune";
    const URL: &'static str = "https://t.me/vst_ntune";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    #[allow(unused_variables)]
    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        #[cfg(windows)]
        {
            Some(Box::new(editor::GoogleEditor {
                url: self.params.url.clone(),
            }))
        }
        #[cfg(not(windows))]
        {
            None
        }
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        ProcessStatus::Normal // transparent pass-through
    }
}

impl Vst3Plugin for NGoogle {
    const VST3_CLASS_ID: [u8; 16] = *b"NGoogleVstNtune!";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_vst3!(NGoogle);
