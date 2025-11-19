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

/// An EPS color (`[R, G, B]`).
///
/// <div class="warning">
///
/// Each value must be in the range of 0.0 to 1.0.
///
/// </div>
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Color(pub [f64; 3]);

impl Pixel for Color {
    type Image = String;
    type Canvas = Canvas;

    #[inline]
    fn default_color(color: ModuleColor) -> Self {
        Self(color.select(Default::default(), [1.0; 3]))
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

    #[inline]
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
            fgr = dark_pixel.0[0],
            fgg = dark_pixel.0[1],
            fgb = dark_pixel.0[2],
            bgr = light_pixel.0[0],
            bgg = light_pixel.0[1],
            bgb = light_pixel.0[2]
        );
        Self { eps, height }
    }

    #[inline]
    fn draw_dark_pixel(&mut self, x: u32, y: u32) {
        self.draw_dark_rect(x, y, 1, 1);
    }

    #[inline]
    fn draw_dark_rect(&mut self, left: u32, top: u32, width: u32, height: u32) {
        let bottom = self.height - top;
        writeln!(self.eps, "{left} {bottom} {width} {height} rectfill")
            .expect("dark rectangle should be drawn");
    }

    #[inline]
    fn into_image(mut self) -> Self::Image {
        self.eps.push_str("%%EOF");
        self.eps
    }
}
