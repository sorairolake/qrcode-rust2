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

use csscolorparser::ParseColorError;

use crate::{
    render::{Canvas as RenderCanvas, Pixel},
    types::Color as ModuleColor,
};

/// An SVG color.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Color<'a>(&'a str);

impl<'a> Color<'a> {
    /// Creates a new `Color` with the given CSS color string.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if `color` does not comply with the W3C's [CSS Color
    /// Module Level 4].
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::render::svg::Color;
    /// #
    /// assert!(Color::new("brown").is_ok());
    /// assert!(Color::new("#111").is_ok());
    ///
    /// assert!(Color::new("rgb(0)").is_err());
    /// assert!(Color::new("unknown").is_err());
    /// ```
    ///
    /// [CSS Color Module Level 4]: https://www.w3.org/TR/css-color-4/
    pub fn new(color: &'a str) -> Result<Self, ParseColorError> {
        csscolorparser::parse(color)?;
        Ok(Self(color))
    }
}

impl<'a> Pixel for Color<'a> {
    type Image = String;
    type Canvas = Canvas<'a>;

    fn default_color(color: ModuleColor) -> Self {
        let color = color.select("#000", "#fff");
        Self::new(color).unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        assert!(Color::new("brown").is_ok());

        assert!(Color::new("#a52a2a").is_ok());
        assert!(Color::new("a52a2a").is_ok());
        assert!(Color::new("#a52a2a7f").is_ok());
        assert!(Color::new("a52a2a7f").is_ok());
        assert!(Color::new("#111").is_ok());
        assert!(Color::new("111").is_ok());
        assert!(Color::new("#1118").is_ok());
        assert!(Color::new("1118").is_ok());
        assert_eq!(Color::new("#g").unwrap_err(), ParseColorError::InvalidHex);

        assert!(Color::new("rgb(165 42 42)").is_ok());
        assert!(Color::new("rgb(165, 42, 42)").is_ok());
        assert!(Color::new("rgb(165 42 42 / 49.8%)").is_ok());
        assert!(Color::new("rgb(165, 42, 42, 49.8%)").is_ok());
        assert!(Color::new("rgba(165 42 42 / 49.8%)").is_ok());
        assert!(Color::new("rgba(165, 42, 42, 49.8%)").is_ok());
        assert_eq!(
            Color::new("rgb(0)").unwrap_err(),
            ParseColorError::InvalidRgb
        );
        assert_eq!(
            Color::new("rgba(0)").unwrap_err(),
            ParseColorError::InvalidRgb
        );

        assert!(Color::new("hsl(248 39% 39.2%)").is_ok());
        assert!(Color::new("hsl(248, 39%, 39.2%)").is_ok());
        assert!(Color::new("hsl(248 39% 39.2% / 49.8%)").is_ok());
        assert!(Color::new("hsl(248, 39%, 39.2%, 49.8%)").is_ok());
        assert!(Color::new("hsla(248 39% 39.2% / 49.8%)").is_ok());
        assert!(Color::new("hsla(248, 39%, 39.2%, 49.8%)").is_ok());
        assert_eq!(
            Color::new("hsl(0)").unwrap_err(),
            ParseColorError::InvalidHsl
        );
        assert_eq!(
            Color::new("hsla(0)").unwrap_err(),
            ParseColorError::InvalidHsl
        );

        assert!(Color::new("hwb(50.6 0% 0%)").is_ok());
        assert!(Color::new("hwb(50.6 0% 0% / 49.8%)").is_ok());
        assert_eq!(
            Color::new("hwb(0)").unwrap_err(),
            ParseColorError::InvalidHwb
        );

        assert!(Color::new("oklab(50.4% -0.0906 0.0069)").is_ok());
        assert!(Color::new("oklab(50.4% -0.0906 0.0069 / 0.5)").is_ok());
        assert_eq!(
            Color::new("oklab(0)").unwrap_err(),
            ParseColorError::InvalidOklab
        );

        assert!(Color::new("oklch(59.41% 0.16 301.29)").is_ok());
        assert!(Color::new("oklch(59.41% 0.16 301.29 / 49.8%)").is_ok());
        assert_eq!(
            Color::new("oklch(0)").unwrap_err(),
            ParseColorError::InvalidOklch
        );

        assert_eq!(
            Color::new("fn(0)").unwrap_err(),
            ParseColorError::InvalidFunction
        );

        assert_eq!(
            Color::new("a").unwrap_err(),
            ParseColorError::InvalidUnknown
        );
    }
}
