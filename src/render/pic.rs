// SPDX-FileCopyrightText: 2024 Alexis Hildebrandt
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
// SPDX-FileCopyrightText: 2024 kennytm
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! [PIC] rendering support.
//!
//! # Examples
//!
//! ```
//! use qrcode2::{QrCode, render::pic::Color};
//!
//! let code = QrCode::new(b"Hello").unwrap();
//! let pic = code.render::<Color>().build();
//! println!("{pic}");
//! ```
//!
//! [PIC]: https://en.wikipedia.org/wiki/PIC_(markup_language)

use alloc::{format, string::String};
use core::fmt::Write;

use crate::{
    render::{Canvas as RenderCanvas, Pixel},
    types::Color as ModuleColor,
};

/// A PIC color.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Color;

impl Pixel for Color {
    type Image = String;
    type Canvas = Canvas;

    #[inline]
    fn default_color(_color: ModuleColor) -> Self {
        Self
    }
}

/// A canvas for PIC rendering.
#[derive(Debug)]
pub struct Canvas {
    pic: String,
}

impl RenderCanvas for Canvas {
    type Pixel = Color;
    type Image = String;

    #[inline]
    fn new(width: u32, height: u32, _dark_pixel: Self::Pixel, _light_pixel: Self::Pixel) -> Self {
        let pic = format!(
            concat!(
                "maxpswid={w};maxpsht={h};movewid=0;moveht=1;boxwid=1;boxht=1\n",
                "define p {{ box wid $3 ht $4 fill 1 thickness 0.1 with .nw at $1,-$2 }}\n",
                "box wid maxpswid ht maxpsht with .nw at 0,0\n"
            ),
            w = width,
            h = height
        );
        Self { pic }
    }

    #[inline]
    fn draw_dark_pixel(&mut self, x: u32, y: u32) {
        self.draw_dark_rect(x, y, 1, 1);
    }

    #[inline]
    fn draw_dark_rect(&mut self, left: u32, top: u32, width: u32, height: u32) {
        writeln!(self.pic, "p({left},{top},{width},{height})")
            .expect("dark rectangle should be drawn");
    }

    #[inline]
    fn into_image(mut self) -> Self::Image {
        self.pic.pop();
        self.pic
    }
}
