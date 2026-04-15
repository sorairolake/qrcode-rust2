// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! [EPS] rendering support.
//!
//! # Examples
//!
//! ```
//! use qrcode2::{QrCode, render::eps::Color};
//!
//! let code = QrCode::new(b"Hello").unwrap();
//! let eps = code.render::<Color>().build();
//! println!("{eps}");
//! ```
//!
//! [EPS]: https://en.wikipedia.org/wiki/Encapsulated_PostScript

use alloc::{format, string::String};
use core::fmt::Write;

use crate::{
    render::{Canvas as RenderCanvas, Pixel},
    types::Color as ModuleColor,
};

/// An EPS color.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Color {
    red: f32,
    green: f32,
    blue: f32,
}

impl Color {
    /// Creates a new `Color` with the given RGB values, returning [`None`] if
    /// any of the values are not between 0.0 and 1.0.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::render::eps::Color;
    /// #
    /// assert!(Color::new(0.0, 0.0, 0.0).is_some());
    /// assert!(Color::new(0.0, 0.5, 1.0).is_some());
    ///
    /// assert!(Color::new(1.1, 0.0, 0.0).is_none());
    /// assert!(Color::new(0.0, -0.1, 0.0).is_none());
    /// assert!(Color::new(0.0, 0.0, f32::NAN).is_none());
    /// ```
    #[must_use]
    pub fn new(red: f32, green: f32, blue: f32) -> Option<Self> {
        if [red, green, blue]
            .into_iter()
            .all(|c| (0.0..=1.0).contains(&c))
        {
            Some(Self { red, green, blue })
        } else {
            None
        }
    }
}

impl Pixel for Color {
    type Image = String;
    type Canvas = Canvas;

    fn default_color(color: ModuleColor) -> Self {
        let [red, green, blue] = color.select(Default::default(), [1.0; 3]);
        Self::new(red, green, blue).unwrap()
    }
}

/// A canvas for EPS rendering.
#[derive(Debug)]
pub struct Canvas {
    eps: String,
    height: u32,
}

impl RenderCanvas for Canvas {
    type Pixel = Color;
    type Image = String;

    fn new(width: u32, height: u32, dark_pixel: Self::Pixel, light_pixel: Self::Pixel) -> Self {
        let eps = format!(
            concat!(
                "%!PS-Adobe-3.0 EPSF-3.0\n",
                "%%BoundingBox: 0 0 {w} {h}\n",
                "%%Pages: 1\n",
                "%%EndComments\n",
                "gsave\n",
                "{bgr} {bgg} {bgb} setrgbcolor\n",
                "0 0 {w} {h} rectfill\n",
                "grestore\n",
                "{fgr} {fgg} {fgb} setrgbcolor\n"
            ),
            w = width,
            h = height,
            fgr = dark_pixel.red,
            fgg = dark_pixel.green,
            fgb = dark_pixel.blue,
            bgr = light_pixel.red,
            bgg = light_pixel.green,
            bgb = light_pixel.blue
        );
        Self { eps, height }
    }

    fn draw_dark_pixel(&mut self, x: u32, y: u32) {
        self.draw_dark_rect(x, y, 1, 1);
    }

    fn draw_dark_rect(&mut self, left: u32, top: u32, width: u32, height: u32) {
        let bottom = self
            .height
            .checked_sub(top)
            .and_then(|y| y.checked_sub(height))
            .unwrap();
        writeln!(self.eps, "{left} {bottom} {width} {height} rectfill").unwrap();
    }

    fn into_image(mut self) -> Self::Image {
        self.eps.push_str("%%EOF");
        self.eps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        assert!(Color::new(0.0, 0.0, 0.0).is_some());
        assert!(Color::new(1.0, 1.0, 1.0).is_some());
        assert!(Color::new(0.0, 0.5, 1.0).is_some());

        assert!(Color::new(1.1, 0.5, 0.5).is_none());
        assert!(Color::new(0.5, 1.1, 0.5).is_none());
        assert!(Color::new(0.5, 0.5, 1.1).is_none());

        assert!(Color::new(-0.1, 0.5, 0.5).is_none());
        assert!(Color::new(0.5, -0.1, 0.5).is_none());
        assert!(Color::new(0.5, 0.5, -0.1).is_none());

        assert!(Color::new(f32::NAN, 0.5, 0.5).is_none());
        assert!(Color::new(0.5, f32::NAN, 0.5).is_none());
        assert!(Color::new(0.5, 0.5, f32::NAN).is_none());

        assert!(Color::new(f32::INFINITY, 0.5, 0.5).is_none());
        assert!(Color::new(0.5, f32::INFINITY, 0.5).is_none());
        assert!(Color::new(0.5, 0.5, f32::INFINITY).is_none());

        assert!(Color::new(f32::NEG_INFINITY, 0.5, 0.5).is_none());
        assert!(Color::new(0.5, f32::NEG_INFINITY, 0.5).is_none());
        assert!(Color::new(0.5, 0.5, f32::NEG_INFINITY).is_none());
    }
}
