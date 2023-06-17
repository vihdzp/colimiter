/*
Copyright © 2023 Violeta Hernández Palacios

This file is part of Colimiter.

Colimiter is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, version 3 of the License.

Colimiter is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Colimiter. If not, see <https://www.gnu.org/licenses/>.
*/

use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::{atomic::Ordering, Arc};

mod editor;
mod icon;

/// My own homepage.
const HOMEPAGE: &str = "https://viiii.neocities.org";

/// The default threshold in decibels.
const DEFAULT_THRESHOLD: f32 = -45.0;

/// The parameters for the plugin.
#[derive(Params)]
struct ColimiterParams {
    /// The editor state.
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    /// The threshold for the colimiter. Stored as a value in decibels.
    #[id = "thresh"]
    pub threshold: FloatParam,
}

impl Default for ColimiterParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            threshold: FloatParam::new(
                "Threshold",
                DEFAULT_THRESHOLD,
                FloatRange::Linear {
                    min: -90.0,
                    max: 20.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(25.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
        }
    }
}

/// The Colimiter plugin.
pub struct Colimiter {
    /// The parameters for the plugin.
    params: Arc<ColimiterParams>,

    /// Peak meter, shared between the GUI and audio processing threads. Stored
    /// as a value in decibels.
    peak_meter: Arc<AtomicF32>,
}

impl Default for Colimiter {
    fn default() -> Self {
        Self {
            params: Arc::new(ColimiterParams::default()),
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
        }
    }
}

impl Plugin for Colimiter {
    const NAME: &'static str = "Colimiter";
    const VENDOR: &'static str = "viiii";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "vi.hdz.p@gmail.com";
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

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.peak_meter.clone())
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut peak = 0.0;

        // The actual audio processing.
        for channel_samples in buffer.iter_samples() {
            let threshold = util::db_to_gain(self.params.threshold.smoothed.next());
            for sample in channel_samples {
                let sample_abs = sample.abs();

                if sample_abs > threshold {
                    if *sample > 0.0 {
                        *sample -= threshold;
                    } else {
                        *sample += threshold;
                    }
                } else {
                    *sample = 0.0;
                }

                // Update peak.
                if sample_abs > peak {
                    peak = sample_abs;
                }
            }

            // Only update the peak meter if visible.
            if self.params.editor_state.is_open() {
                self.peak_meter.store(peak, Ordering::Relaxed);
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Colimiter {
    const CLAP_ID: &'static str = "com.viiii.colimiter";
    const CLAP_DESCRIPTION: Option<&'static str> = Some(env!("CARGO_PKG_DESCRIPTION"));
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for Colimiter {
    const VST3_CLASS_ID: [u8; 16] = *b"viiii-colimiter.";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(Colimiter);
