/*
Copyright © 2023 Violeta Hernández Palacios

This file is part of Colimiter.

Colimiter is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, version 3 of the License.

Colimiter is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Colimiter. If not, see <https://www.gnu.org/licenses/>. 
*/

use std::cell::RefCell;

use nih_plug_vizia::vizia;
use vg::rgb::FromSlice;
use vizia::image;
use vizia::prelude::*;
use vizia::vg;

/// The size for the Colimiter icon.
pub const IMG_SIZE: usize = 50;

/// The Colimiter icon.
pub struct ColimiterIcon {
    /// A handle to the image.
    image: RefCell<Option<vg::ImageId>>,
}

impl ColimiterIcon {
    /// Builds the Colimiter icon.
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {
            image: RefCell::new(None),
        }
        .build(cx, |_| {})
    }
}

/// Returns the Colimiter logo, which is embedded in the executable.
fn logo() -> image::ImageResult<image::RgbaImage> {
    let mut reader = image::io::Reader::new(std::io::Cursor::new(include_bytes!(
        "../resources/Logo.png"
    )));
    reader.set_format(image::ImageFormat::Png);
    reader.decode().map(|img| img.into_rgba8())
}

impl View for ColimiterIcon {
    fn element(&self) -> Option<&'static str> {
        Some("functor-icon")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        // Gets the handle to the logo.
        let image_id = if self.image.borrow().is_none() {
            let logo = logo().expect("logo failed to load");
            let image = vg::imgref::Img::new(logo.as_rgba(), IMG_SIZE, IMG_SIZE);
            let image_id = canvas
                .create_image(image, vg::ImageFlags::empty())
                .expect("logo couldn't be loaded");

            *self.image.borrow_mut() = Some(image_id);
            image_id
        } else {
            self.image.borrow().unwrap()
        };

        // Draw the logo in a square area.
        let mut path = vg::Path::new();
        path.rect(bounds.x, bounds.y, bounds.w, bounds.h);
        let paint = vg::Paint::image(image_id, bounds.x, bounds.y, bounds.w, bounds.h, 0.0, 1.0);
        canvas.fill_path(&mut path, &paint);
    }
}
