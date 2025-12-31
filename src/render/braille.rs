// SPDX-FileCopyrightText: 2025 Mattias Jansson
//
// SPDX-License-Identifier: Apache-2.0 OR MIT
//! UTF-8 braille rendering, with 8 pixels per symbol.
use super::{Canvas, Pixel};

type BrailleImage = String;

const HEIGHT_PER_BYTE: u32 = 4;
const WIDTH_PER_BYTE: u32 = 2;

/// An image pixel for braille rendering.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BraillePixel {
    /// The pixel is light colored.
    Light = 0,

    /// The pixel is dark colored.
    Dark = 1,
}

impl Pixel for BraillePixel {
    type Image = BrailleImage;
    type Canvas = BrailleCanvas;

    fn default_color(color: crate::Color) -> Self {
        color.select(Self::Dark, Self::Light)
    }

    fn default_unit_size() -> (u32, u32) {
        (1, 1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BrailleByte {
    pixels: u8,
}

impl BrailleByte {
    const fn new(fill_color: BraillePixel) -> Self {
        Self {
            pixels: u8::MAX * fill_color as u8,
        }
    }

    const fn set(&mut self, x: u32, y: u32, color: BraillePixel) {
        match color {
            BraillePixel::Light => self.pixels &= !Self::pattern(x, y),
            BraillePixel::Dark => self.pixels |= Self::pattern(x, y),
        }
    }

    const fn pattern(x: u32, y: u32) -> u8 {
        // The characters have a bit of an irregular encoding
        // so we skip the math
        match (y, x) {
            (0, 0) => 0b0000_0001,
            (1, 0) => 0b0000_0010,
            (2, 0) => 0b0000_0100,
            (3, 0) => 0b0100_0000,
            (0, 1) => 0b0000_1000,
            (1, 1) => 0b0001_0000,
            (2, 1) => 0b0010_0000,
            (3, 1) => 0b1000_0000,
            _ => 0, // This would be a bug
        }
    }

    fn to_char(&self) -> char {
        char::from_u32(0x2800 + u32::from(self.pixels))
            .expect("Invalid UTF-8 braille character. This is a bug. Please report it")
    }
}

/// A canvas for UTF-8 rendering with braille characters giving a resolution of
/// 2×4 modules per character.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrailleCanvas {
    buffer: Vec<BrailleByte>,
    byte_width: u32,
    dark_pixel: BraillePixel,
}

impl BrailleCanvas {
    fn draw_pixel(&mut self, x: u32, y: u32, pixel: BraillePixel) {
        let byte_x = x / WIDTH_PER_BYTE;
        let byte_y = y / HEIGHT_PER_BYTE;
        let stored = &mut self.buffer[(byte_x + byte_y * self.byte_width) as usize];

        let bit_x = x % WIDTH_PER_BYTE;
        let bit_y = y % HEIGHT_PER_BYTE;
        stored.set(bit_x, bit_y, pixel);
    }

    fn clear_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        for y in y..(y + height) {
            for x in x..(x + width) {
                self.draw_pixel(x, y, BraillePixel::Light);
            }
        }
    }
}

impl Canvas for BrailleCanvas {
    type Pixel = BraillePixel;
    type Image = BrailleImage;

    fn new(width: u32, height: u32, dark_pixel: BraillePixel, light_pixel: BraillePixel) -> Self {
        let byte_width = width.div_ceil(WIDTH_PER_BYTE);
        let byte_height = height.div_ceil(HEIGHT_PER_BYTE);

        let buffer = vec![BrailleByte::new(light_pixel); (byte_width * byte_height) as usize];

        let mut canvas = Self {
            buffer,
            byte_width,
            dark_pixel,
        };

        // Clear any bits overflowing the intended size
        let canvas_width = byte_width * WIDTH_PER_BYTE;
        let canvas_height = byte_height * HEIGHT_PER_BYTE;
        canvas.clear_rect(width, 0, canvas_width - width, canvas_height);
        canvas.clear_rect(0, height, canvas_width, canvas_height - height);
        canvas
    }

    fn draw_dark_pixel(&mut self, x: u32, y: u32) {
        self.draw_pixel(x, y, self.dark_pixel);
    }

    fn into_image(self) -> Self::Image {
        self.buffer
            .chunks_exact(self.byte_width as usize)
            .map(|row| row.iter().map(BrailleByte::to_char).collect())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[test]
fn test_render_to_utf8_string() {
    use crate::render::{Color, Renderer};
    let colors = &[Color::Dark, Color::Light, Color::Light, Color::Dark];
    let image: String = Renderer::<BraillePixel>::new(colors, 2, 2, 1).build();

    assert_eq!(&image, "⠐⠄");

    let image2 = Renderer::<BraillePixel>::new(colors, 2, 2, 1)
        .module_dimensions(2, 2)
        .build();

    assert_eq!(&image2, "⠀⣤⠀⠀\n⠀⠀⠛⠀");
}

#[test]
fn integration_render_utf8_1x2() {
    use crate::{EcLevel, QrCode, Version};

    let code = QrCode::with_version(b"09876542", Version::Micro(2), EcLevel::L).unwrap();
    let image = code
        .render::<BraillePixel>()
        .module_dimensions(1, 1)
        .build();
    assert_eq!(
        image,
        String::new()
            + "⠀⡤⠤⠤⡄⠄⡄⠄⠀\n"
            + "⠀⡇⠿⠇⡇⠨⡜⡄⠀\n"
            + "⠀⢭⠩⠭⠥⣮⠥⡃⠀\n"
            + "⠀⠽⠟⠆⠭⠸⠘⠇⠀\n"
            + "⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    );
}

#[test]
fn integration_render_utf8_1x2_inverted() {
    use crate::{EcLevel, QrCode, Version};

    let code = QrCode::with_version(b"12345678", Version::Micro(2), EcLevel::L).unwrap();
    let image = code
        .render::<BraillePixel>()
        .dark_color(BraillePixel::Light)
        .light_color(BraillePixel::Dark)
        .module_dimensions(1, 1)
        .build();
    assert_eq!(
        image,
        "⣿⢛⣛⣛⢻⡻⣻⣻⡇\n\
         ⣿⢸⣀⣸⢸⣀⣼⢼⡇\n\
         ⣿⡒⠖⠖⢞⡒⢪⣼⡇\n\
         ⣿⣒⣱⣃⣍⣥⣱⣺⡇\n\
         ⠉⠉⠉⠉⠉⠉⠉⠉⠁"
    );
}
