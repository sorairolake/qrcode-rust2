// SPDX-FileCopyrightText: 2017 kennytm
// SPDX-FileCopyrightText: 2020 Vladimir Serov
// SPDX-FileCopyrightText: 2021 CodeAssemblingChicken
// SPDX-FileCopyrightText: 2024 Alexis Hildebrandt
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Render a QR code into image.

#[cfg(feature = "eps")]
pub mod eps;
#[cfg(feature = "image")]
pub mod image;
#[cfg(feature = "pic")]
pub mod pic;
pub mod string;
#[cfg(feature = "svg")]
pub mod svg;
pub mod unicode;

use core::cmp;

use crate::{cast::As, types::Color};

/// Abstraction of an image pixel.
pub trait Pixel: Copy + Sized {
    /// Type of the finalized image.
    type Image: Sized + 'static;

    /// The type that stores an intermediate buffer before finalizing to a
    /// concrete image.
    type Canvas: Canvas<Pixel = Self, Image = Self::Image>;

    /// Obtains the default module size. The result must be at least 1×1.
    #[must_use]
    fn default_unit_size() -> (u32, u32) {
        (8, 8)
    }

    /// Obtains the default pixel color when a module is dark or light.
    fn default_color(color: Color) -> Self;
}

/// Rendering canvas of a QR code image.
pub trait Canvas: Sized {
    /// Type of an image pixel.
    type Pixel: Sized;

    /// Type of the finalized image.
    type Image: Sized;

    /// Constructs a new canvas of the given dimensions.
    fn new(width: u32, height: u32, dark_pixel: Self::Pixel, light_pixel: Self::Pixel) -> Self;

    /// Draws a single dark pixel at the (x, y) coordinate.
    fn draw_dark_pixel(&mut self, x: u32, y: u32);

    /// Draws a dark rectangle with dimensions `width`×`height` at the (`left`,
    /// `top`) coordinate.
    fn draw_dark_rect(&mut self, left: u32, top: u32, width: u32, height: u32) {
        for y in top..(top + height) {
            for x in left..(left + width) {
                self.draw_dark_pixel(x, y);
            }
        }
    }

    /// Finalizes the canvas to a real image.
    fn into_image(self) -> Self::Image;
}

/// A QR code renderer. This is a builder type which converts a bool-vector into
/// an image.
#[derive(Debug)]
pub struct Renderer<'a, P: Pixel> {
    content: &'a [Color],
    // we call it `horizontal_modules_count` here to avoid ambiguity of `width`.
    horizontal_modules_count: u32,
    // we call it `vertical_modules_count` here to avoid ambiguity of `height`.
    vertical_modules_count: u32,
    quiet_zone: u32,
    module_size: (u32, u32),
    dark_color: P,
    light_color: P,
    has_quiet_zone: bool,
}

impl<'a, P: Pixel> Renderer<'a, P> {
    /// Creates a new renderer.
    ///
    /// Except for rMQR code, `horizontal_modules_count` and
    /// `vertical_modules_count` should be the same value.
    ///
    /// # Panics
    ///
    /// Panics if the length of `content` is not exactly
    /// `horizontal_modules_count * vertical_modules_count`.
    #[must_use]
    pub fn new(
        content: &'a [Color],
        horizontal_modules_count: usize,
        vertical_modules_count: usize,
        quiet_zone: u32,
    ) -> Self {
        assert_eq!(
            horizontal_modules_count * vertical_modules_count,
            content.len()
        );
        let horizontal_modules_count = horizontal_modules_count.as_u32();
        let vertical_modules_count = vertical_modules_count.as_u32();
        let module_size = P::default_unit_size();
        let dark_color = P::default_color(Color::Dark);
        let light_color = P::default_color(Color::Light);
        Self {
            content,
            horizontal_modules_count,
            vertical_modules_count,
            quiet_zone,
            module_size,
            dark_color,
            light_color,
            has_quiet_zone: true,
        }
    }

    /// Sets color of a dark module. Default is opaque black.
    pub const fn dark_color(&mut self, color: P) -> &mut Self {
        self.dark_color = color;
        self
    }

    /// Sets color of a light module. Default is opaque white.
    pub const fn light_color(&mut self, color: P) -> &mut Self {
        self.light_color = color;
        self
    }

    /// Sets the size of the quiet zone in the generated image.
    ///
    /// If `Renderer` is constructed using
    /// [`QrCode::render`](crate::QrCode::render), the size of the quiet zone is
    /// 4 for QR code model 2, and 2 for Micro QR code and rMQR code.
    pub const fn quiet_zone(&mut self, quiet_zone: u32) -> &mut Self {
        self.quiet_zone = quiet_zone;
        self
    }

    /// Sets whether to include the quiet zone in the generated image.
    pub const fn has_quiet_zone(&mut self, has_quiet_zone: bool) -> &mut Self {
        self.has_quiet_zone = has_quiet_zone;
        self
    }

    /// Sets the size of each module in pixels. Default is 8×8.
    pub fn module_dimensions(&mut self, width: u32, height: u32) -> &mut Self {
        self.module_size = (cmp::max(width, 1), cmp::max(height, 1));
        self
    }

    /// Sets the minimum total image size in pixels, including the quiet zone if
    /// applicable. The renderer will try to find the dimension as small as
    /// possible, such that each module in the QR code has uniform size (no
    /// distortion).
    ///
    /// For instance, a version 1 QR code has 19 modules across including the
    /// quiet zone. If we request an image of size ≥200×200, we get that each
    /// module's size should be 11×11, so the actual image size will be 209×209.
    pub fn min_dimensions(&mut self, width: u32, height: u32) -> &mut Self {
        let quiet_zone = if self.has_quiet_zone { 2 } else { 0 } * self.quiet_zone;
        let width_in_modules = self.horizontal_modules_count + quiet_zone;
        let height_in_modules = self.vertical_modules_count + quiet_zone;
        let unit_width = width.div_ceil(width_in_modules);
        let unit_height = height.div_ceil(height_in_modules);
        self.module_dimensions(unit_width, unit_height)
    }

    /// Sets the maximum total image size in pixels, including the quiet zone if
    /// applicable. The renderer will try to find the dimension as large as
    /// possible, such that each module in the QR code has uniform size (no
    /// distortion).
    ///
    /// For instance, a version 1 QR code has 19 modules across including the
    /// quiet zone. If we request an image of size ≤200×200, we get that each
    /// module's size should be 10×10, so the actual image size will be 190×190.
    ///
    /// The module size is at least 1×1, so if the restriction is too small, the
    /// final image _can_ be larger than the input.
    pub fn max_dimensions(&mut self, width: u32, height: u32) -> &mut Self {
        let quiet_zone = if self.has_quiet_zone { 2 } else { 0 } * self.quiet_zone;
        let width_in_modules = self.horizontal_modules_count + quiet_zone;
        let height_in_modules = self.vertical_modules_count + quiet_zone;
        let unit_width = width / width_in_modules;
        let unit_height = height / height_in_modules;
        self.module_dimensions(unit_width, unit_height)
    }

    /// Renders the QR code into an image.
    pub fn build(&self) -> P::Image {
        let w = self.horizontal_modules_count;
        let h = self.vertical_modules_count;
        let qz = if self.has_quiet_zone {
            self.quiet_zone
        } else {
            0
        };
        let width = w + 2 * qz;
        let height = h + 2 * qz;

        let (mw, mh) = self.module_size;
        let real_width = width * mw;
        let real_height = height * mh;

        let mut canvas = P::Canvas::new(real_width, real_height, self.dark_color, self.light_color);
        let mut i = 0;
        for y in 0..height {
            for x in 0..width {
                if qz <= x && x < w + qz && qz <= y && y < h + qz {
                    if self.content[i] != Color::Light {
                        canvas.draw_dark_rect(x * mw, y * mh, mw, mh);
                    }
                    i += 1;
                }
            }
        }

        canvas.into_image()
    }
}
