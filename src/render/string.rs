// SPDX-FileCopyrightText: 2017 Jovansonlee Cesar
// SPDX-FileCopyrightText: 2017 kennytm
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! String rendering support.
//!
//! # Examples
//!
//! ```
//! use qrcode2::QrCode;
//!
//! let code = QrCode::new(b"Hello").unwrap();
//! let s = code.render::<char>().build();
//! println!("{s}");
//! ```

use alloc::{string::String, vec, vec::Vec};

use crate::{
    cast::As,
    render::{Canvas as RenderCanvas, Pixel},
    types::Color,
};

/// Abstraction of an image element.
pub trait Element: Copy {
    /// Obtains the default element color when a module is dark or light.
    fn default_color(color: Color) -> Self;

    /// Returns the number of bytes in `self`.
    fn str_len(self) -> usize;

    /// Appends `self` to the end of the given `string`.
    fn append_to_string(self, string: &mut String);
}

impl Element for char {
    fn default_color(color: Color) -> Self {
        color.select('\u{2588}', ' ')
    }

    fn str_len(self) -> usize {
        self.len_utf8()
    }

    fn append_to_string(self, string: &mut String) {
        string.push(self);
    }
}

impl Element for &str {
    fn default_color(color: Color) -> Self {
        color.select("\u{2588}", " ")
    }

    fn str_len(self) -> usize {
        self.len()
    }

    fn append_to_string(self, string: &mut String) {
        string.push_str(self);
    }
}

/// A canvas for string rendering.
#[derive(Debug)]
pub struct Canvas<P: Element> {
    buffer: Vec<P>,
    width: usize,
    dark_pixel: P,
    dark_cap_inc: isize,
    capacity: isize,
}

impl<P: Element> Pixel for P {
    type Image = String;
    type Canvas = Canvas<Self>;

    fn default_unit_size() -> (u32, u32) {
        (1, 1)
    }

    fn default_color(color: Color) -> Self {
        <Self as Element>::default_color(color)
    }
}

impl<P: Element> RenderCanvas for Canvas<P> {
    type Pixel = P;
    type Image = String;

    fn new(width: u32, height: u32, dark_pixel: Self::Pixel, light_pixel: Self::Pixel) -> Self {
        let width = width.as_usize();
        let height = isize::try_from(height).unwrap();
        let buffer = vec![light_pixel; height.as_usize() * width];
        let dark_cap = dark_pixel.str_len().try_into().unwrap();
        let light_cap = light_pixel.str_len().try_into().unwrap();
        let dark_cap_inc = dark_cap - light_cap;
        let capacity = light_cap * width.try_into().unwrap() * height + (height - 1);
        Self {
            buffer,
            width,
            dark_pixel,
            dark_cap_inc,
            capacity,
        }
    }

    fn draw_dark_pixel(&mut self, x: u32, y: u32) {
        let x = x.as_usize();
        let y = y.as_usize();
        self.capacity += self.dark_cap_inc;
        self.buffer[x + y * self.width] = self.dark_pixel;
    }

    fn into_image(self) -> Self::Image {
        let mut result = String::with_capacity(self.capacity.as_usize());
        for (i, pixel) in self.buffer.into_iter().enumerate() {
            if i != 0 && i.is_multiple_of(self.width) {
                result.push('\n');
            }
            pixel.append_to_string(&mut result);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::Renderer;

    #[test]
    fn render_to_string() {
        let colors = &[Color::Dark, Color::Light, Color::Light, Color::Dark];
        let image: String = Renderer::<char>::new(colors, 2, 2, 1).build();
        assert_eq!(
            &image,
            concat!("    \n", " \u{2588}  \n", "  \u{2588} \n", "    ")
        );

        let image2 = Renderer::new(colors, 2, 2, 1)
            .light_color("A")
            .dark_color("!B!")
            .module_dimensions(2, 2)
            .build();

        assert_eq!(
            &image2,
            concat!(
                "AAAAAAAA\n",
                "AAAAAAAA\n",
                "AA!B!!B!AAAA\n",
                "AA!B!!B!AAAA\n",
                "AAAA!B!!B!AA\n",
                "AAAA!B!!B!AA\n",
                "AAAAAAAA\n",
                "AAAAAAAA"
            )
        );
    }
}
