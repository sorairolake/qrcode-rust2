// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of features related to the mask patterns.

use super::Canvas;
use crate::types::{EcLevel, Version};

/// The mask patterns. Since QR code and Micro QR code do not use the same
/// pattern number, we name them according to their shape instead of the number.
#[derive(Clone, Copy, Debug)]
pub enum MaskPattern {
    /// QR code mask pattern `0b000`.
    Checkerboard = 0b000,

    /// QR code mask pattern `0b001`, and Micro QR code mask pattern `0b00`.
    HorizontalLines = 0b001,

    /// QR code mask pattern `0b010`.
    VerticalLines = 0b010,

    /// QR code mask pattern `0b011`.
    DiagonalLines = 0b011,

    /// QR code mask pattern `0b100`, and Micro QR code mask pattern `0b01`.
    LargeCheckerboard = 0b100,

    /// QR code mask pattern `0b101`.
    Fields = 0b101,

    /// QR code mask pattern `0b110`, and Micro QR code mask pattern `0b10`.
    Diamonds = 0b110,

    /// QR code mask pattern `0b111`, and Micro QR code mask pattern `0b11`.
    Meadow = 0b111,
}

mod mask_functions {
    pub const fn checkerboard(x: i16, y: i16) -> bool {
        (x + y) % 2 == 0
    }

    pub const fn horizontal_lines(_: i16, y: i16) -> bool {
        y % 2 == 0
    }

    pub const fn vertical_lines(x: i16, _: i16) -> bool {
        x % 3 == 0
    }

    pub const fn diagonal_lines(x: i16, y: i16) -> bool {
        (x + y) % 3 == 0
    }

    pub const fn large_checkerboard(x: i16, y: i16) -> bool {
        ((y / 2) + (x / 3)) % 2 == 0
    }

    pub const fn fields(x: i16, y: i16) -> bool {
        (x * y) % 2 + (x * y) % 3 == 0
    }

    pub const fn diamonds(x: i16, y: i16) -> bool {
        ((x * y) % 2 + (x * y) % 3) % 2 == 0
    }

    pub const fn meadow(x: i16, y: i16) -> bool {
        ((x + y) % 2 + (x * y) % 3) % 2 == 0
    }
}

fn get_mask_function(pattern: MaskPattern) -> fn(i16, i16) -> bool {
    match pattern {
        MaskPattern::Checkerboard => mask_functions::checkerboard,
        MaskPattern::HorizontalLines => mask_functions::horizontal_lines,
        MaskPattern::VerticalLines => mask_functions::vertical_lines,
        MaskPattern::DiagonalLines => mask_functions::diagonal_lines,
        MaskPattern::LargeCheckerboard => mask_functions::large_checkerboard,
        MaskPattern::Fields => mask_functions::fields,
        MaskPattern::Diamonds => mask_functions::diamonds,
        MaskPattern::Meadow => mask_functions::meadow,
    }
}

impl Canvas {
    /// Applies a mask to the canvas. This method will also draw the format info
    /// patterns.
    pub fn apply_mask(&mut self, pattern: MaskPattern) {
        let mask_fn = get_mask_function(pattern);
        for x in 0..self.width {
            for y in 0..self.height {
                let module = self.get_mut(x, y);
                *module = module.mask(mask_fn(x, y));
            }
        }

        self.draw_format_info_patterns(pattern);
    }

    /// Draws the format information to encode the error correction level and
    /// mask pattern.
    ///
    /// If the error correction level or mask pattern is not supported in the
    /// current QR code version, this method will fail.
    fn draw_format_info_patterns(&mut self, pattern: MaskPattern) {
        if self.version.is_rect_micro() {
            return;
        }

        let format_number = match self.version {
            Version::Normal(_) => {
                let simple_format_number = ((self.ec_level as usize) ^ 1) << 3 | (pattern as usize);
                FORMAT_INFOS_QR[simple_format_number]
            }
            Version::Micro(a) => {
                let micro_pattern_number = match pattern {
                    MaskPattern::HorizontalLines => 0b00,
                    MaskPattern::LargeCheckerboard => 0b01,
                    MaskPattern::Diamonds => 0b10,
                    MaskPattern::Meadow => 0b11,
                    _ => panic!("Unsupported mask pattern in Micro QR code"),
                };
                let symbol_number = match (a, self.ec_level) {
                    (1, EcLevel::L) => 0b000,
                    (2, EcLevel::L) => 0b001,
                    (2, EcLevel::M) => 0b010,
                    (3, EcLevel::L) => 0b011,
                    (3, EcLevel::M) => 0b100,
                    (4, EcLevel::L) => 0b101,
                    (4, EcLevel::M) => 0b110,
                    (4, EcLevel::Q) => 0b111,
                    _ => panic!("Unsupported version/ec_level combination in Micro QR code"),
                };
                let simple_format_number = symbol_number << 2 | micro_pattern_number;
                FORMAT_INFOS_MICRO_QR[simple_format_number]
            }
            Version::RectMicro(..) => return,
        };
        self.draw_format_info_patterns_with_number(format_number);
    }
}

static FORMAT_INFOS_QR: [u16; 32] = [
    0x5412, 0x5125, 0x5E7C, 0x5B4B, 0x45F9, 0x40CE, 0x4F97, 0x4AA0, 0x77C4, 0x72F3, 0x7DAA, 0x789D,
    0x662F, 0x6318, 0x6C41, 0x6976, 0x1689, 0x13BE, 0x1CE7, 0x19D0, 0x0762, 0x0255, 0x0D0C, 0x083B,
    0x355F, 0x3068, 0x3F31, 0x3A06, 0x24B4, 0x2183, 0x2EDA, 0x2BED,
];

static FORMAT_INFOS_MICRO_QR: [u16; 32] = [
    0x4445, 0x4172, 0x4E2B, 0x4B1C, 0x55AE, 0x5099, 0x5FC0, 0x5AF7, 0x6793, 0x62A4, 0x6DFD, 0x68CA,
    0x7678, 0x734F, 0x7C16, 0x7921, 0x06DE, 0x03E9, 0x0CB0, 0x0987, 0x1735, 0x1202, 0x1D5B, 0x186C,
    0x2508, 0x203F, 0x2F66, 0x2A51, 0x34E3, 0x31D4, 0x3E8D, 0x3BBA,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_mask_qr() {
        let mut c = Canvas::new(Version::Normal(1), EcLevel::L);
        c.draw_all_functional_patterns();
        c.apply_mask(MaskPattern::Checkerboard);

        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######...#.#.#######\n",
                "#.....#..#.#..#.....#\n",
                "#.###.#.#.#.#.#.###.#\n",
                "#.###.#..#.#..#.###.#\n",
                "#.###.#...#.#.#.###.#\n",
                "#.....#..#.#..#.....#\n",
                "#######.#.#.#.#######\n",
                "........##.#.........\n",
                "###.#####.#.###...#..\n",
                ".#.#.#.#.#.#.#.#.#.#.\n",
                "#.#.#.#.#.#.#.#.#.#.#\n",
                ".#.#.#.#.#.#.#.#.#.#.\n",
                "#.#.#.#.#.#.#.#.#.#.#\n",
                "........##.#.#.#.#.#.\n",
                "#######.#.#.#.#.#.#.#\n",
                "#.....#.##.#.#.#.#.#.\n",
                "#.###.#.#.#.#.#.#.#.#\n",
                "#.###.#..#.#.#.#.#.#.\n",
                "#.###.#.#.#.#.#.#.#.#\n",
                "#.....#.##.#.#.#.#.#.\n",
                "#######.#.#.#.#.#.#.#"
            )
        );
    }

    #[test]
    fn test_draw_format_info_patterns_qr() {
        let mut c = Canvas::new(Version::Normal(1), EcLevel::L);
        c.draw_format_info_patterns(MaskPattern::LargeCheckerboard);
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "????????#????????????\n",
                "????????#????????????\n",
                "????????#????????????\n",
                "????????#????????????\n",
                "????????.????????????\n",
                "????????#????????????\n",
                "?????????????????????\n",
                "????????.????????????\n",
                "##..##?..????..#.####\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "????????#????????????\n",
                "????????.????????????\n",
                "????????#????????????\n",
                "????????#????????????\n",
                "????????.????????????\n",
                "????????.????????????\n",
                "????????#????????????\n",
                "????????#????????????"
            )
        );
    }

    #[test]
    fn test_draw_format_info_patterns_micro_qr() {
        let mut c = Canvas::new(Version::Micro(2), EcLevel::L);
        c.draw_format_info_patterns(MaskPattern::LargeCheckerboard);
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "?????????????\n",
                "????????#????\n",
                "????????.????\n",
                "????????.????\n",
                "????????#????\n",
                "????????#????\n",
                "????????.????\n",
                "????????.????\n",
                "?#.#....#????\n",
                "?????????????\n",
                "?????????????\n",
                "?????????????\n",
                "?????????????"
            )
        );
    }
}
