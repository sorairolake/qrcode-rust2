// SPDX-FileCopyrightText: 2017 kennytm
// SPDX-FileCopyrightText: 2019 Atul Bhosale
// SPDX-FileCopyrightText: 2019 Jasper Bryant-Greene
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Raster image rendering support powered by the [`image`] crate.
//!
//! # Examples
//!
//! ```
//! use qrcode2::{QrCode, image::Luma};
//!
//! let code = QrCode::new(b"Hello").unwrap();
//! let image = code.render::<Luma<u8>>().build();
//! let temp_dir = tempfile::tempdir().unwrap();
//! image.save(temp_dir.path().join("qrcode.png")).unwrap();
//! ```

use alloc::vec::Vec;

use image::{ImageBuffer, Luma, LumaA, Primitive, Rgb, Rgba};

use crate::{
    render::{Canvas, Pixel},
    types::Color,
};

impl<S> Pixel for Luma<S>
where
    S: Primitive + 'static,
    Self: image::Pixel<Subpixel = S>,
{
    type Image = ImageBuffer<Self, Vec<S>>;
    type Canvas = (Self, Self::Image);

    fn default_color(color: Color) -> Self {
        let p = color.select(S::zero(), S::max_value());
        Self([p])
    }
}

impl<S> Pixel for LumaA<S>
where
    S: Primitive + 'static,
    Self: image::Pixel<Subpixel = S>,
{
    type Image = ImageBuffer<Self, Vec<S>>;
    type Canvas = (Self, Self::Image);

    fn default_color(color: Color) -> Self {
        let p = color.select(S::zero(), S::max_value());
        Self([p, S::max_value()])
    }
}

impl<S> Pixel for Rgb<S>
where
    S: Primitive + 'static,
    Self: image::Pixel<Subpixel = S>,
{
    type Image = ImageBuffer<Self, Vec<S>>;
    type Canvas = (Self, Self::Image);

    fn default_color(color: Color) -> Self {
        let p = color.select(S::zero(), S::max_value());
        Self([p, p, p])
    }
}

impl<S> Pixel for Rgba<S>
where
    S: Primitive + 'static,
    Self: image::Pixel<Subpixel = S>,
{
    type Image = ImageBuffer<Self, Vec<S>>;
    type Canvas = (Self, Self::Image);

    fn default_color(color: Color) -> Self {
        let p = color.select(S::zero(), S::max_value());
        Self([p, p, p, S::max_value()])
    }
}

impl<P: image::Pixel + 'static> Canvas for (P, ImageBuffer<P, Vec<P::Subpixel>>) {
    type Pixel = P;
    type Image = ImageBuffer<P, Vec<P::Subpixel>>;

    fn new(width: u32, height: u32, dark_pixel: Self::Pixel, light_pixel: Self::Pixel) -> Self {
        (
            dark_pixel,
            ImageBuffer::from_pixel(width, height, light_pixel),
        )
    }

    fn draw_dark_pixel(&mut self, x: u32, y: u32) {
        self.1.put_pixel(x, y, self.0);
    }

    fn into_image(self) -> Self::Image {
        self.1
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;
    use crate::render::Renderer;

    #[test]
    fn test_render_luma8_unsized() {
        let image = Renderer::<Luma<u8>>::new(
            &[
                Color::Light,
                Color::Dark,
                Color::Dark,
                //
                Color::Dark,
                Color::Light,
                Color::Light,
                //
                Color::Light,
                Color::Dark,
                Color::Light,
            ],
            3,
            3,
            1,
        )
        .module_dimensions(1, 1)
        .build();

        let expected = [
            255, 255, 255, 255, 255, 255, 255, 0, 0, 255, 255, 0, 255, 255, 255, 255, 255, 0, 255,
            255, 255, 255, 255, 255, 255,
        ];
        assert_eq!(image.into_raw(), expected);
    }

    #[test]
    fn test_render_rgba_unsized() {
        let image = Renderer::<Rgba<u8>>::new(
            &[Color::Light, Color::Dark, Color::Dark, Color::Dark],
            2,
            2,
            1,
        )
        .module_dimensions(1, 1)
        .build();

        let expected: &[u8] = &[
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255,
        ];

        assert_eq!(image.into_raw(), expected);
    }

    #[test]
    fn test_render_resized_min() {
        let image = Renderer::<Luma<u8>>::new(
            &[Color::Dark, Color::Light, Color::Light, Color::Dark],
            2,
            2,
            1,
        )
        .min_dimensions(10, 10)
        .build();

        let expected: &[u8] = &[
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0,
            0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        ];

        assert_eq!(image.dimensions(), (12, 12));
        assert_eq!(image.into_raw(), expected);
    }

    #[test]
    fn test_render_resized_max() {
        let image = Renderer::<Luma<u8>>::new(
            &[Color::Dark, Color::Light, Color::Light, Color::Dark],
            2,
            2,
            1,
        )
        .max_dimensions(10, 5)
        .build();

        let expected: &[u8] = &[
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            255, 255, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        ];

        assert_eq!(image.dimensions(), (8, 4));
        assert_eq!(image.into_raw(), expected);
    }
}
