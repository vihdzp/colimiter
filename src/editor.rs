/*
Copyright © 2023 Violeta Hernández Palacios

This file is part of Colimiter.

Colimiter is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, version 3 of the License.

Colimiter is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Colimiter. If not, see <https://www.gnu.org/licenses/>. 
*/

use atomic_float::AtomicF32;
use nih_plug::{
    prelude::{Editor, Plugin},
    util,
};
use nih_plug_vizia::{
    vizia::prelude::*,
    widgets::{ParamSlider, PeakMeter},
    *,
};
use std::sync::{atomic::Ordering, Arc};

use crate::{icon, Colimiter, ColimiterParams};

/// The size of the window.
const SIZE: (u32, u32) = (400, 220);

/// Color of the header background.
const HEADER_BG: Color = Color::rgb(150, 200, 255);

/// Color for lesser accented text.
const GRAY: Color = Color::rgb(50, 50, 50);

/// The default window state.
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| SIZE)
}

/// Tries to open a webpage. If it fails (which apparently can happen on
/// Windows), we ignore the error.
fn open_webpage(path: &str) {
    let result = open::that(path);
    if cfg!(debug) && result.is_err() {
        crate::nih_debug_assert_failure!("Failed to open web browser: {:?}", result);
    }
}

/// The Colimiter editor.
#[derive(Lens)]
struct ColimiterEditor {
    /// The parameters for the plugin.
    params: Arc<ColimiterParams>,

    /// Peak meter, shared between the GUI and audio processing threads. Stored
    /// as a value in decibels.
    peak_meter: Arc<AtomicF32>,
}

impl Model for ColimiterEditor {}

/// Contains all the layout logic.
pub(crate) fn create(
    params: Arc<ColimiterParams>,
    peak_meter: Arc<AtomicF32>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(
        params.editor_state.clone(),
        ViziaTheming::Custom,
        move |cx, _| {
            assets::register_noto_sans_light(cx);

            ColimiterEditor {
                params: params.clone(),
                peak_meter: peak_meter.clone(),
            }
            .build(cx);

            // Header
            HStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    // Colimiter icon
                    crate::icon::ColimiterIcon::new(cx)
                        .size(Pixels(icon::IMG_SIZE as f32))
                        .top(Pixels(12.5))
                        .bottom(Pixels(12.5))
                        .left(Pixels(15.0))
                        .right(Pixels(12.5));

                    // Colimiter label
                    Label::new(cx, "Colimiter").font_size(52.0).top(Pixels(4.0));

                    // Version number
                    Label::new(cx, Colimiter::VERSION)
                        .font_size(20.0)
                        .top(Pixels(36.0))
                        .color(GRAY);
                });

                // Help button
                Label::new(cx, "?")
                    .border_color(Color::black())
                    .border_radius(Pixels(10.0))
                    .border_width(Pixels(1.0))
                    .font_size(16.0)
                    .size(Pixels(20.0))
                    .child_left(Pixels(6.0))
                    .top(Pixels(3.0))
                    .right(Pixels(3.0))
                    .cursor(CursorIcon::Hand) // Broken in baseview.
                    .on_mouse_down(|_, _| {
                        open_webpage(Colimiter::URL);
                    });
            })
            .bottom(Pixels(10.0))
            .background_color(HEADER_BG)
            .max_height(Pixels(icon::IMG_SIZE as f32));

            // Other controls
            VStack::new(cx, |cx| {
                // Threshold param slider
                Label::new(cx, "Threshold");
                ParamSlider::new(cx, ColimiterEditor::params, |params| &params.threshold)
                    .bottom(Pixels(10.0));

                // Peak meter
                Label::new(cx, "Peak Meter");
                PeakMeter::new(
                    cx,
                    ColimiterEditor::peak_meter
                        .map(|peak_meter| util::gain_to_db(peak_meter.load(Ordering::Relaxed))),
                    Some(std::time::Duration::from_millis(100)),
                )
                .bottom(Pixels(10.0));
            })
            .row_between(Pixels(0.0))
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0));

            // Copyright label
            Label::new(cx, "© Violeta Hernández Palacios (2023)")
                .font_size(10.0)
                .left(Pixels(5.0))
                .color(GRAY)
                .on_mouse_down(|_, _| {
                    open_webpage(crate::HOMEPAGE);
                });
        },
    )
}
