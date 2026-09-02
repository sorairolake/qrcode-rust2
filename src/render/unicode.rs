// SPDX-FileCopyrightText: 2020 Vladimir Serov
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! UTF-8 rendering, with 2 pixels per symbol.
//!
//! # Examples
//!
//! ```
//! use qrcode2::{QrCode, render::unicode::Dense1x2};
//!
//! let code = QrCode::new(b"Hello").unwrap();
//! let s = code.render::<Dense1x2>().build();
//! println!("{s}");
//! ```

use alloc::{string::String, vec, vec::Vec};

use crate::{
    cast::As,
    render::{Canvas as RenderCanvas, Color, Pixel},
};

const CODEPAGE: [char; 4] = [' ', '\u{2584}', '\u{2580}', '\u{2588}'];

/// An image pixel for UTF-8 rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dense1x2 {
    /// The pixel is dark colored.
    Dark,

    /// The pixel is light colored.
    Light,
}

impl Pixel for Dense1x2 {
    type Image = String;
    type Canvas = Canvas1x2;

    fn default_color(color: Color) -> Self {
        color.select(Self::Dark, Self::Light)
    }

    fn default_unit_size() -> (u32, u32) {
        (1, 1)
    }
}

impl Dense1x2 {
    const fn value(self) -> u8 {
        match self {
            Self::Dark => 1,
            Self::Light => 0,
        }
    }

    fn parse_2_bits(sym: u8) -> char {
        CODEPAGE[usize::from(sym)]
    }
}

/// A canvas for UTF-8 rendering with a resolution of 1×2 modules per character.
#[derive(Debug)]
pub struct Canvas1x2 {
    canvas: Vec<u8>,
    width: u32,
    dark_pixel: u8,
}

impl RenderCanvas for Canvas1x2 {
    type Pixel = Dense1x2;
    type Image = String;

    fn new(width: u32, height: u32, dark_pixel: Self::Pixel, light_pixel: Self::Pixel) -> Self {
        let canvas = vec![light_pixel.value(); (width * height).as_usize()];
        let dark_pixel = dark_pixel.value();
        Self {
            canvas,
            width,
            dark_pixel,
        }
    }

    fn draw_dark_pixel(&mut self, x: u32, y: u32) {
        self.canvas[(x + y * self.width).as_usize()] = self.dark_pixel;
    }

    fn into_image(self) -> Self::Image {
        let width = self.width.as_usize();
        let mut result = String::new();

        // Chopping the array into 2-line chunks.
        let mut iter = self.canvas.chunks(width * 2).peekable();

        while let Some(chunk) = iter.next() {
            let top_row = chunk.get(..width).unwrap_or(chunk);
            let bottom_row = chunk.get(width..);

            // Zipping those 2 lines together into 2-bit values.
            for (x, top) in top_row.iter().enumerate() {
                let bottom = bottom_row
                    .and_then(|r| r.get(x))
                    .copied()
                    .unwrap_or_default();
                let bits = (top << 1) | bottom;

                // Mapping those 2-bit numbers to corresponding pixels.
                result.push(Dense1x2::parse_2_bits(bits));
            }
            if iter.peek().is_some() {
                result.push('\n');
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        render::Renderer,
        types::{EcLevel, MicroVersion, QrCode, Version},
    };

    #[test]
    fn render_to_utf8_string() {
        let colors = &[Color::Dark, Color::Light, Color::Light, Color::Dark];
        let image: String = Renderer::<Dense1x2>::new(colors, 2, 2, 1).build();

        assert_eq!(&image, concat!(" ▄  \n", "  ▀ "));

        let image2 = Renderer::<Dense1x2>::new(colors, 2, 2, 1)
            .module_dimensions(2, 2)
            .build();

        assert_eq!(
            &image2,
            concat!("        \n", "  ██    \n", "    ██  \n", "        ")
        );
    }

    #[test]
    fn integration_render_utf8_1x2() {
        let code = QrCode::with_version(b"09876542", Version::Micro(MicroVersion::M2), EcLevel::L)
            .unwrap();
        let image = code.render::<Dense1x2>().module_dimensions(1, 1).build();
        assert_eq!(
            image,
            concat!(
                "                 \n",
                "  █▀▀▀▀▀█ ▀ █ ▀  \n",
                "  █ ███ █  ▀ █   \n",
                "  █ ▀▀▀ █  ▀█ █  \n",
                "  ▀▀▀▀▀▀▀ ▄▀▀ █  \n",
                "  ▀█ ▀▀▀▀▀██▀▀▄  \n",
                "  ▀███▄ ▀▀ █ ██  \n",
                "  ▀▀▀ ▀ ▀▀ ▀  ▀  \n",
                "                 "
            )
        );
    }

    #[test]
    fn integration_render_utf8_1x2_inverted() {
        let code = QrCode::with_version(b"12345678", Version::Micro(MicroVersion::M2), EcLevel::L)
            .unwrap();
        let image = code
            .render::<Dense1x2>()
            .dark_color(Dense1x2::Light)
            .light_color(Dense1x2::Dark)
            .module_dimensions(1, 1)
            .build();
        assert_eq!(
            image,
            concat!(
                "█████████████████\n",
                "██ ▄▄▄▄▄ █▄▀▄█▄██\n",
                "██ █   █ █   █ ██\n",
                "██ █▄▄▄█ █▄▄██▀██\n",
                "██▄▄▄▄▄▄▄█▄▄▄▀ ██\n",
                "██▄ ▀ ▀ ▀▄▄  ████\n",
                "██▄▄▀▄█ ▀▀▀ ▀▄▄██\n",
                "██▄▄▄█▄▄█▄██▄█▄██\n",
                "▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀"
            )
        );
    }
}
