// SPDX-FileCopyrightText: 2017 kennytm
// SPDX-FileCopyrightText: 2018 Kyle Clemens
// SPDX-FileCopyrightText: 2019 Markus Kohlhase
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! [SVG] rendering support.
//!
//! # Examples
//!
//! ```
//! use qrcode2::{QrCode, render::svg::Color};
//!
//! let code = QrCode::new(b"Hello").unwrap();
//! let svg_xml = code.render::<Color>().build();
//! println!("{svg_xml}");
//! ```
//!
//! [SVG]: https://www.w3.org/Graphics/SVG/

use alloc::{format, string::String};
use core::{fmt::Write, marker::PhantomData};

use crate::{
    render::{Canvas as RenderCanvas, Pixel},
    types::Color as ModuleColor,
};

/// An SVG color.
///
/// <div class="warning">
///
/// The color value must comply with the W3C's [CSS Color Module Level 4].
///
/// </div>
///
/// [CSS Color Module Level 4]: https://www.w3.org/TR/css-color-4/
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Color<'a>(pub &'a str);

impl<'a> Pixel for Color<'a> {
    type Image = String;
    type Canvas = Canvas<'a>;

    fn default_color(color: ModuleColor) -> Self {
        Color(color.select("#000", "#fff"))
    }
}

/// A canvas for SVG rendering.
#[derive(Debug)]
pub struct Canvas<'a> {
    svg: String,
    marker: PhantomData<Color<'a>>,
}

impl<'a> RenderCanvas for Canvas<'a> {
    type Pixel = Color<'a>;
    type Image = String;

    fn new(width: u32, height: u32, dark_pixel: Self::Pixel, light_pixel: Self::Pixel) -> Self {
        let svg = format!(
            concat!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
                r#"<svg xmlns="http://www.w3.org/2000/svg""#,
                r#" version="1.1" width="{w}" height="{h}""#,
                r#" viewBox="0 0 {w} {h}" shape-rendering="crispEdges">"#,
                r#"<path d="M0 0h{w}v{h}H0z" fill="{bg}"/>"#,
                r#"<path fill="{fg}" d=""#
            ),
            w = width,
            h = height,
            fg = dark_pixel.0,
            bg = light_pixel.0
        );
        Self {
            svg,
            marker: PhantomData,
        }
    }

    fn draw_dark_pixel(&mut self, x: u32, y: u32) {
        self.draw_dark_rect(x, y, 1, 1);
    }

    fn draw_dark_rect(&mut self, left: u32, top: u32, width: u32, height: u32) {
        write!(self.svg, "M{left} {top}h{width}v{height}h-{width}z").unwrap();
    }

    fn into_image(mut self) -> Self::Image {
        self.svg.push_str(r#""/></svg>"#);
        self.svg
    }
}
