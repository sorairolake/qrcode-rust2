// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2018 Ignas Anikevicius
// SPDX-FileCopyrightText: 2019 Atul Bhosale
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The `canvas` module puts raw bits into the QR code canvas.
//!
//! # Examples
//!
//! ```
//! use qrcode2::{
//!     EcLevel, Version,
//!     canvas::{Canvas, MaskPattern},
//! };
//!
//! let mut c = Canvas::new(Version::Normal(1), EcLevel::L);
//! c.draw_all_functional_patterns();
//! c.draw_data(b"data_here", b"ec_code_here");
//! c.apply_mask(MaskPattern::Checkerboard);
//! let colors = c.into_colors();
//! ```

#[cfg(test)]
use alloc::string::String;
use alloc::{boxed::Box, vec, vec::Vec};
use core::{cmp, iter};

use crate::{
    cast::As,
    types::{Color, EcLevel, Version},
};

// Modules

/// The color of a module (pixel) in the QR code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Module {
    /// The module is empty.
    Empty,

    /// The module is of functional patterns which cannot be masked, or pixels
    /// which have been masked.
    Masked(Color),

    /// The module is of data and error correction bits before masking.
    Unmasked(Color),
}

impl From<Module> for Color {
    fn from(module: Module) -> Self {
        match module {
            Module::Empty => Self::Light,
            Module::Masked(c) | Module::Unmasked(c) => c,
        }
    }
}

impl Module {
    /// Checks whether a module is dark.
    #[must_use]
    pub fn is_dark(self) -> bool {
        Color::from(self) == Color::Dark
    }

    /// Applies a mask to the unmasked modules.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::{Color, canvas::Module};
    /// #
    /// assert_eq!(
    ///     Module::Unmasked(Color::Light).mask(true),
    ///     Module::Masked(Color::Dark)
    /// );
    /// assert_eq!(
    ///     Module::Unmasked(Color::Dark).mask(true),
    ///     Module::Masked(Color::Light)
    /// );
    /// assert_eq!(
    ///     Module::Unmasked(Color::Light).mask(false),
    ///     Module::Masked(Color::Light)
    /// );
    /// assert_eq!(
    ///     Module::Masked(Color::Dark).mask(true),
    ///     Module::Masked(Color::Dark)
    /// );
    /// assert_eq!(
    ///     Module::Masked(Color::Dark).mask(false),
    ///     Module::Masked(Color::Dark)
    /// );
    /// ```
    #[must_use]
    pub fn mask(self, should_invert: bool) -> Self {
        match (self, should_invert) {
            (Self::Empty, true) => Self::Masked(Color::Dark),
            (Self::Empty, false) => Self::Masked(Color::Light),
            (Self::Unmasked(c), true) => Self::Masked(!c),
            (Self::Unmasked(c), false) | (Self::Masked(c), _) => Self::Masked(c),
        }
    }
}

// Canvas

/// `Canvas` is an intermediate helper structure to render error-corrected data
/// into a QR code.
#[derive(Clone, Debug)]
pub struct Canvas {
    /// The width of the canvas (cached as it is needed frequently).
    width: i16,

    /// The height of the canvas (cached as it is needed frequently).
    height: i16,

    /// The version of the QR code.
    version: Version,

    /// The error correction level of the QR code.
    ec_level: EcLevel,

    /// The modules of the QR code. Modules are arranged in left-to-right, then
    /// top-to-bottom order.
    modules: Vec<Module>,
}

impl Canvas {
    /// Constructs a new canvas big enough for a QR code of the given version.
    #[must_use]
    pub fn new(version: Version, ec_level: EcLevel) -> Self {
        let (width, height) = (version.width(), version.height());
        let modules = vec![Module::Empty; (width * height).as_usize()];
        Self {
            width,
            height,
            version,
            ec_level,
            modules,
        }
    }

    /// Converts the canvas into a human-readable string.
    #[cfg(test)]
    fn to_debug_str(&self) -> String {
        let width = self.width;
        let mut res = String::with_capacity((width * (width + 1)) as usize);
        for y in 0..self.height {
            res.push('\n');
            for x in 0..width {
                res.push(match self.get(x, y) {
                    Module::Empty => '?',
                    Module::Masked(Color::Light) => '.',
                    Module::Masked(Color::Dark) => '#',
                    Module::Unmasked(Color::Light) => '-',
                    Module::Unmasked(Color::Dark) => '*',
                });
            }
        }
        res
    }

    /// Converts the canvas into a human-readable string.
    #[cfg(test)]
    fn to_debug_str_mask_same(&self) -> String {
        let width = self.width;
        let mut res = String::with_capacity((width * (width + 1)) as usize);
        for y in 0..self.height {
            res.push('\n');
            for x in 0..width {
                res.push(match self.get(x, y) {
                    Module::Empty => '?',
                    Module::Masked(Color::Light) => '.',
                    Module::Masked(Color::Dark) => '#',
                    Module::Unmasked(Color::Light) => '.',
                    Module::Unmasked(Color::Dark) => '#',
                });
            }
        }
        res
    }

    fn coords_to_index(&self, x: i16, y: i16) -> usize {
        let x = if x < 0 { x + self.width } else { x }.as_usize();
        let y = if y < 0 { y + self.height } else { y }.as_usize();
        y * self.width.as_usize() + x
    }

    /// Obtains a module at the given coordinates. For convenience, negative
    /// coordinates will wrap around.
    #[must_use]
    pub fn get(&self, x: i16, y: i16) -> Module {
        self.modules[self.coords_to_index(x, y)]
    }

    /// Obtains a mutable module at the given coordinates. For convenience,
    /// negative coordinates will wrap around.
    pub fn get_mut(&mut self, x: i16, y: i16) -> &mut Module {
        let index = self.coords_to_index(x, y);
        &mut self.modules[index]
    }

    /// Sets the color of a functional module at the given coordinates. For
    /// convenience, negative coordinates will wrap around.
    pub fn put(&mut self, x: i16, y: i16, color: Color) {
        *self.get_mut(x, y) = Module::Masked(color);
    }
}

#[cfg(test)]
mod basic_canvas_tests {
    use super::*;

    #[test]
    fn test_index() {
        let mut c = Canvas::new(Version::Normal(1), EcLevel::L);

        assert_eq!(c.get(0, 4), Module::Empty);
        assert_eq!(c.get(-1, -7), Module::Empty);
        assert_eq!(c.get(21 - 1, 21 - 7), Module::Empty);

        c.put(0, 0, Color::Dark);
        c.put(-1, -7, Color::Light);
        assert_eq!(c.get(0, 0), Module::Masked(Color::Dark));
        assert_eq!(c.get(21 - 1, -7), Module::Masked(Color::Light));
        assert_eq!(c.get(-1, 21 - 7), Module::Masked(Color::Light));
    }

    #[test]
    fn test_debug_str() {
        let mut c = Canvas::new(Version::Normal(1), EcLevel::L);

        for i in 3_i16..20 {
            for j in 3_i16..20 {
                *c.get_mut(i, j) = match ((i * 3) ^ j) % 5 {
                    0 => Module::Empty,
                    1 => Module::Masked(Color::Light),
                    2 => Module::Masked(Color::Dark),
                    3 => Module::Unmasked(Color::Light),
                    4 => Module::Unmasked(Color::Dark),
                    _ => unreachable!(),
                };
            }
        }

        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????####****....---?\n",
                "???--.##-..##?..#??.?\n",
                "???#*?-.*?#.-*#?-*.??\n",
                "?????*?*?****-*-*---?\n",
                "???*.-.-.-?-?#?#?#*#?\n",
                "???.*#.*.*#.*#*#.*#*?\n",
                "?????.#-#--??.?.#---?\n",
                "???-.?*.-#?-.?#*-#?.?\n",
                "???##*??*..##*--*..??\n",
                "?????-???--??---?---?\n",
                "???*.#.*.#**.#*#.#*#?\n",
                "???##.-##..##..?#..??\n",
                "???.-?*.-?#.-?#*-?#*?\n",
                "????-.#?-.**#?-.#?-.?\n",
                "???**?-**??--**?-**??\n",
                "???#?*?#?*#.*-.-*-.-?\n",
                "???..-...--??###?###?\n",
                "?????????????????????"
            )
        );
    }
}

// Finder patterns

impl Canvas {
    /// Draws a single finder pattern with the center at (x, y).
    fn draw_finder_pattern_at(&mut self, x: i16, y: i16) {
        let (dx_left, dx_right) = if x >= 0 { (-3, 4) } else { (-4, 3) };
        let (dy_top, dy_bottom) = if self.height == 7 {
            (-3, 3)
        } else if y >= 0 {
            (-3, 4)
        } else {
            (-4, 3)
        };
        for j in dy_top..=dy_bottom {
            for i in dx_left..=dx_right {
                self.put(
                    x + i,
                    y + j,
                    match (i, j) {
                        (4 | -4, _) | (_, 4 | -4) => Color::Light,
                        (3 | -3, _) | (_, 3 | -3) => Color::Dark,
                        (2 | -2, _) | (_, 2 | -2) => Color::Light,
                        _ => Color::Dark,
                    },
                );
            }
        }
    }

    /// Draws a single finder pattern for rMQR code.
    ///
    /// In rMQR code, there is one finder pattern that has the same shape as the
    /// alignment pattern located in the bottom right corner.
    fn draw_finder_pattern_rmqr_at(&mut self) {
        self.draw_alignment_pattern_at(self.width - 3, self.height - 3);
    }

    /// Draws the finder patterns.
    ///
    /// The finder patterns is are 7×7 square patterns appearing at the three
    /// corners of a QR code. They allows scanner to locate the QR code and
    /// determine the orientation.
    fn draw_finder_patterns(&mut self) {
        self.draw_finder_pattern_at(3, 3);

        match self.version {
            Version::Micro(_) => {}
            Version::Normal(_) => {
                self.draw_finder_pattern_at(-4, 3);
                self.draw_finder_pattern_at(3, -4);
            }
            Version::RectMicro(..) => self.draw_finder_pattern_rmqr_at(),
        }
    }
}

#[cfg(test)]
mod finder_pattern_tests {
    use super::*;

    #[test]
    fn test_qr() {
        let mut c = Canvas::new(Version::Normal(1), EcLevel::L);
        c.draw_finder_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######.?????.#######\n",
                "#.....#.?????.#.....#\n",
                "#.###.#.?????.#.###.#\n",
                "#.###.#.?????.#.###.#\n",
                "#.###.#.?????.#.###.#\n",
                "#.....#.?????.#.....#\n",
                "#######.?????.#######\n",
                "........?????........\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "........?????????????\n",
                "#######.?????????????\n",
                "#.....#.?????????????\n",
                "#.###.#.?????????????\n",
                "#.###.#.?????????????\n",
                "#.###.#.?????????????\n",
                "#.....#.?????????????\n",
                "#######.?????????????"
            )
        );
    }

    #[test]
    fn test_micro_qr() {
        let mut c = Canvas::new(Version::Micro(1), EcLevel::L);
        c.draw_finder_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######.???\n",
                "#.....#.???\n",
                "#.###.#.???\n",
                "#.###.#.???\n",
                "#.###.#.???\n",
                "#.....#.???\n",
                "#######.???\n",
                "........???\n",
                "???????????\n",
                "???????????\n",
                "???????????"
            )
        );
    }

    #[test]
    fn test_rmqr() {
        let mut c = Canvas::new(Version::RectMicro(7, 43), EcLevel::M);
        c.draw_finder_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######.???????????????????????????????????\n",
                "#.....#.???????????????????????????????????\n",
                "#.###.#.??????????????????????????????#####\n",
                "#.###.#.??????????????????????????????#...#\n",
                "#.###.#.??????????????????????????????#.#.#\n",
                "#.....#.??????????????????????????????#...#\n",
                "#######.??????????????????????????????#####"
            )
        );
    }
}

// Alignment patterns

impl Canvas {
    /// Draws a alignment pattern with the center at (x, y).
    fn draw_alignment_pattern_at(&mut self, x: i16, y: i16) {
        if self.get(x, y) != Module::Empty {
            return;
        }
        for j in -2..=2 {
            for i in -2..=2 {
                self.put(
                    x + i,
                    y + j,
                    match (i, j) {
                        (2 | -2, _) | (_, 2 | -2) | (0, 0) => Color::Dark,
                        _ => Color::Light,
                    },
                );
            }
        }
    }

    /// Draws a alignment pattern in rMQR code with the center at (x, y).
    fn draw_alignment_pattern_rmqr_at(&mut self, x: i16, y: i16) {
        if self.get(x, y) != Module::Empty {
            return;
        }
        for j in -1..=1 {
            for i in -1..=1 {
                self.put(x + i, y + j, Color::Dark);
            }
        }
        self.put(x, y, Color::Light);
    }

    /// Draws the alignment patterns except for rMQR code.
    ///
    /// The alignment patterns are 5×5 square patterns inside the QR code symbol
    /// to help the scanner create the square grid.
    fn draw_alignment_patterns(&mut self) {
        match self.version {
            Version::Micro(_) | Version::Normal(1) | Version::RectMicro(..) => {}
            Version::Normal(2..=6) => self.draw_alignment_pattern_at(-7, -7),
            Version::Normal(a) => {
                let positions = ALIGNMENT_PATTERN_POSITIONS[(a - 7).as_usize()];
                for x in positions {
                    for y in positions {
                        self.draw_alignment_pattern_at(*x, *y);
                    }
                }
            }
        }
    }

    /// Draws the alignment patterns in rMQR code.
    fn draw_alignment_patterns_rmqr(&mut self) {
        if self.version.is_rect_micro() {
            let index = self.version.rect_micro_width_index().unwrap() + 34;
            let x_positons = ALIGNMENT_PATTERN_POSITIONS[index];
            for x in x_positons {
                self.draw_alignment_pattern_rmqr_at(*x, 1);
                self.draw_alignment_pattern_rmqr_at(*x, self.height - 2);
            }
        }
    }
}

#[cfg(test)]
mod alignment_pattern_tests {
    use super::*;

    #[test]
    fn test_draw_alignment_patterns_1() {
        let mut c = Canvas::new(Version::Normal(1), EcLevel::L);
        c.draw_finder_patterns();
        c.draw_alignment_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######.?????.#######\n",
                "#.....#.?????.#.....#\n",
                "#.###.#.?????.#.###.#\n",
                "#.###.#.?????.#.###.#\n",
                "#.###.#.?????.#.###.#\n",
                "#.....#.?????.#.....#\n",
                "#######.?????.#######\n",
                "........?????........\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "........?????????????\n",
                "#######.?????????????\n",
                "#.....#.?????????????\n",
                "#.###.#.?????????????\n",
                "#.###.#.?????????????\n",
                "#.###.#.?????????????\n",
                "#.....#.?????????????\n",
                "#######.?????????????"
            )
        );
    }

    #[test]
    fn test_draw_alignment_patterns_3() {
        let mut c = Canvas::new(Version::Normal(3), EcLevel::L);
        c.draw_finder_patterns();
        c.draw_alignment_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######.?????????????.#######\n",
                "#.....#.?????????????.#.....#\n",
                "#.###.#.?????????????.#.###.#\n",
                "#.###.#.?????????????.#.###.#\n",
                "#.###.#.?????????????.#.###.#\n",
                "#.....#.?????????????.#.....#\n",
                "#######.?????????????.#######\n",
                "........?????????????........\n",
                "?????????????????????????????\n",
                "?????????????????????????????\n",
                "?????????????????????????????\n",
                "?????????????????????????????\n",
                "?????????????????????????????\n",
                "?????????????????????????????\n",
                "?????????????????????????????\n",
                "?????????????????????????????\n",
                "?????????????????????????????\n",
                "?????????????????????????????\n",
                "?????????????????????????????\n",
                "?????????????????????????????\n",
                "????????????????????#####????\n",
                "........????????????#...#????\n",
                "#######.????????????#.#.#????\n",
                "#.....#.????????????#...#????\n",
                "#.###.#.????????????#####????\n",
                "#.###.#.?????????????????????\n",
                "#.###.#.?????????????????????\n",
                "#.....#.?????????????????????\n",
                "#######.?????????????????????"
            )
        );
    }

    #[test]
    fn test_draw_alignment_patterns_7() {
        let mut c = Canvas::new(Version::Normal(7), EcLevel::L);
        c.draw_finder_patterns();
        c.draw_alignment_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######.?????????????????????????????.#######\n",
                "#.....#.?????????????????????????????.#.....#\n",
                "#.###.#.?????????????????????????????.#.###.#\n",
                "#.###.#.?????????????????????????????.#.###.#\n",
                "#.###.#.????????????#####????????????.#.###.#\n",
                "#.....#.????????????#...#????????????.#.....#\n",
                "#######.????????????#.#.#????????????.#######\n",
                "........????????????#...#????????????........\n",
                "????????????????????#####????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "????#####???????????#####???????????#####????\n",
                "????#...#???????????#...#???????????#...#????\n",
                "????#.#.#???????????#.#.#???????????#.#.#????\n",
                "????#...#???????????#...#???????????#...#????\n",
                "????#####???????????#####???????????#####????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "????????????????????#####???????????#####????\n",
                "........????????????#...#???????????#...#????\n",
                "#######.????????????#.#.#???????????#.#.#????\n",
                "#.....#.????????????#...#???????????#...#????\n",
                "#.###.#.????????????#####???????????#####????\n",
                "#.###.#.?????????????????????????????????????\n",
                "#.###.#.?????????????????????????????????????\n",
                "#.....#.?????????????????????????????????????\n",
                "#######.?????????????????????????????????????"
            )
        );
    }

    #[test]
    fn test_draw_alignment_patterns_rmqr_7x77() {
        let mut c = Canvas::new(Version::RectMicro(7, 77), EcLevel::L);
        c.draw_finder_patterns();
        c.draw_alignment_patterns_rmqr();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######.????????????????###???????????????????????###????????????????????????\n",
                "#.....#.????????????????#.#???????????????????????#.#????????????????????????\n",
                "#.###.#.????????????????###???????????????????????###???????????????????#####\n",
                "#.###.#.????????????????????????????????????????????????????????????????#...#\n",
                "#.###.#.????????????????###???????????????????????###???????????????????#.#.#\n",
                "#.....#.????????????????#.#???????????????????????#.#???????????????????#...#\n",
                "#######.????????????????###???????????????????????###???????????????????#####"
            )
        );
    }
}

/// `ALIGNMENT_PATTERN_POSITIONS` describes the x- and y-coordinates of the
/// center of the alignment patterns. Since the QR code is symmetric, only one
/// coordinate is needed. rMQR code is symmetrically placed at the top and
/// bottom, so only one coordinate is needed.
static ALIGNMENT_PATTERN_POSITIONS: [&[i16]; 40] = [
    &[6, 22, 38],
    &[6, 24, 42],
    &[6, 26, 46],
    &[6, 28, 50],
    &[6, 30, 54],
    &[6, 32, 58],
    &[6, 34, 62],
    &[6, 26, 46, 66],
    &[6, 26, 48, 70],
    &[6, 26, 50, 74],
    &[6, 30, 54, 78],
    &[6, 30, 56, 82],
    &[6, 30, 58, 86],
    &[6, 34, 62, 90],
    &[6, 28, 50, 72, 94],
    &[6, 26, 50, 74, 98],
    &[6, 30, 54, 78, 102],
    &[6, 28, 54, 80, 106],
    &[6, 32, 58, 84, 110],
    &[6, 30, 58, 86, 114],
    &[6, 34, 62, 90, 118],
    &[6, 26, 50, 74, 98, 122],
    &[6, 30, 54, 78, 102, 126],
    &[6, 26, 52, 78, 104, 130],
    &[6, 30, 56, 82, 108, 134],
    &[6, 34, 60, 86, 112, 138],
    &[6, 30, 58, 86, 114, 142],
    &[6, 34, 62, 90, 118, 146],
    &[6, 30, 54, 78, 102, 126, 150],
    &[6, 24, 50, 76, 102, 128, 154],
    &[6, 28, 54, 80, 106, 132, 158],
    &[6, 32, 58, 84, 110, 136, 162],
    &[6, 26, 54, 82, 110, 138, 166],
    &[6, 30, 58, 86, 114, 142, 170],
    // rMQR versions.
    // 27
    &[],
    // 43
    &[21],
    // 59
    &[19, 39],
    // 77
    &[25, 51],
    // 99
    &[23, 49, 75],
    // 139
    &[27, 55, 83, 111],
];

// Corner finder patterns for rMQR code

impl Canvas {
    /// Draws the rMQR corner finder pattern.
    fn draw_corner_finder_pattern(&mut self) {
        if !self.version.is_rect_micro() {
            return;
        }
        // Bottom left
        self.put(0, -1, Color::Dark);
        self.put(1, -1, Color::Dark);
        self.put(2, -1, Color::Dark);

        // Top right
        self.put(-1, 0, Color::Dark);
        self.put(-1, 1, Color::Dark);
        self.put(-2, 0, Color::Dark);
        self.put(-2, 1, Color::Light);

        if self.height >= 11 {
            self.put(0, -2, Color::Dark);
            self.put(1, -2, Color::Light);
        }
    }
}

// Timing patterns

impl Canvas {
    /// Draws a line from (x1, y1) to (x2, y2), inclusively.
    ///
    /// The line must be either horizontal or vertical, i.e. `x1 == x2 || y1 ==
    /// y2`. Additionally, the first coordinates must be less then the second
    /// ones.
    ///
    /// On even coordinates, `color_even` will be plotted; on odd coordinates,
    /// `color_odd` will be plotted instead. Thus the timing pattern can be
    /// drawn using this method.
    fn draw_line(
        &mut self,
        x1: i16,
        y1: i16,
        x2: i16,
        y2: i16,
        color_even: Color,
        color_odd: Color,
    ) {
        debug_assert!(x1 == x2 || y1 == y2);

        if y1 == y2 {
            // Horizontal line.
            for x in x1..=x2 {
                self.put(x, y1, if x % 2 == 0 { color_even } else { color_odd });
            }
        } else {
            // Vertical line.
            for y in y1..=y2 {
                self.put(x1, y, if y % 2 == 0 { color_even } else { color_odd });
            }
        }
    }

    fn draw_rmqr_line(&mut self) {
        let (width, height) = (self.width, self.height);

        // Top.
        self.draw_line(8, 0, width - 3, 0, Color::Dark, Color::Light);

        // Bottom.
        if height == 7 {
            self.draw_line(
                8,
                height - 1,
                width - 6,
                height - 1,
                Color::Dark,
                Color::Light,
            );
        } else {
            self.draw_line(
                3,
                height - 1,
                width - 6,
                height - 1,
                Color::Dark,
                Color::Light,
            );
        }

        // Left.
        if height >= 11 {
            self.draw_line(0, 8, 0, height - 3, Color::Dark, Color::Light);
        }

        // Right.
        if height >= 9 {
            self.draw_line(
                width - 1,
                2,
                width - 1,
                height - 6,
                Color::Dark,
                Color::Light,
            );
        }

        let position_index = self.version.rect_micro_width_index().unwrap() + 34;
        for x in ALIGNMENT_PATTERN_POSITIONS[position_index] {
            self.draw_line(*x, 3, *x, height - 4, Color::Dark, Color::Light);
        }
    }

    /// Draws the timing patterns.
    ///
    /// The timing patterns are checkboard-colored lines near the edge of the QR
    /// code symbol, to establish the fine-grained module coordinates when
    /// scanning.
    fn draw_timing_patterns(&mut self) {
        if let Version::RectMicro(..) = self.version {
            self.draw_rmqr_line();
        } else {
            let width = self.width;
            let (y, x1, x2) = match self.version {
                Version::Micro(_) => (0, 8, width - 1),
                Version::Normal(_) => (6, 8, width - 9),
                Version::RectMicro(..) => unreachable!(),
            };
            self.draw_line(x1, y, x2, y, Color::Dark, Color::Light);
            self.draw_line(y, x1, y, x2, Color::Dark, Color::Light);
        }
    }
}

#[cfg(test)]
mod timing_pattern_tests {
    use super::*;

    #[test]
    fn test_draw_timing_patterns_qr() {
        let mut c = Canvas::new(Version::Normal(1), EcLevel::L);
        c.draw_timing_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "????????#.#.#????????\n",
                "?????????????????????\n",
                "??????#??????????????\n",
                "??????.??????????????\n",
                "??????#??????????????\n",
                "??????.??????????????\n",
                "??????#??????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????"
            )
        );
    }

    #[test]
    fn test_draw_timing_patterns_micro_qr() {
        let mut c = Canvas::new(Version::Micro(1), EcLevel::L);
        c.draw_timing_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "????????#.#\n",
                "???????????\n",
                "???????????\n",
                "???????????\n",
                "???????????\n",
                "???????????\n",
                "???????????\n",
                "???????????\n",
                "#??????????\n",
                ".??????????\n",
                "#??????????"
            )
        );
    }

    #[test]
    fn test_draw_timing_patterns_rmqr_7x77() {
        let mut c = Canvas::new(Version::RectMicro(7, 77), EcLevel::L);
        c.draw_timing_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "????????#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#??\n",
                "?????????????????????????????????????????????????????????????????????????????\n",
                "?????????????????????????????????????????????????????????????????????????????\n",
                "?????????????????????????.?????????????????????????.?????????????????????????\n",
                "?????????????????????????????????????????????????????????????????????????????\n",
                "?????????????????????????????????????????????????????????????????????????????\n",
                "????????#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.?????"
            )
        );
    }

    #[test]
    fn test_draw_timing_patterns_rmqr_9x77() {
        let mut c = Canvas::new(Version::RectMicro(9, 77), EcLevel::L);
        c.draw_timing_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "????????#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#??\n",
                "?????????????????????????????????????????????????????????????????????????????\n",
                "????????????????????????????????????????????????????????????????????????????#\n",
                "?????????????????????????.?????????????????????????.????????????????????????.\n",
                "?????????????????????????#?????????????????????????#?????????????????????????\n",
                "?????????????????????????.?????????????????????????.?????????????????????????\n",
                "?????????????????????????????????????????????????????????????????????????????\n",
                "?????????????????????????????????????????????????????????????????????????????\n",
                "???.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.?????"
            )
        );
    }

    #[test]
    fn test_draw_timing_patterns_rmqr_11x77() {
        let mut c = Canvas::new(Version::RectMicro(11, 77), EcLevel::L);
        c.draw_timing_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "????????#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#??\n",
                "?????????????????????????????????????????????????????????????????????????????\n",
                "????????????????????????????????????????????????????????????????????????????#\n",
                "?????????????????????????.?????????????????????????.????????????????????????.\n",
                "?????????????????????????#?????????????????????????#????????????????????????#\n",
                "?????????????????????????.?????????????????????????.????????????????????????.\n",
                "?????????????????????????#?????????????????????????#?????????????????????????\n",
                "?????????????????????????.?????????????????????????.?????????????????????????\n",
                "#????????????????????????????????????????????????????????????????????????????\n",
                "?????????????????????????????????????????????????????????????????????????????\n",
                "???.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.?????"
            )
        );
    }
}

// Format info & Version info

impl Canvas {
    /// Draws a big-endian integer onto the canvas with the given coordinates.
    ///
    /// The 1 bits will be plotted with `on_color` and the 0 bits with
    /// `off_color`. The coordinates will be extracted from the `coords`
    /// iterator. It will start from the most significant bits first, so
    /// _trailing_ zeros will be ignored.
    fn draw_number(
        &mut self,
        number: u32,
        bits: u32,
        on_color: Color,
        off_color: Color,
        coords: &[(i16, i16)],
    ) {
        let mut mask = 1 << (bits - 1);
        for &(x, y) in coords {
            let color = if (mask & number) == 0 {
                off_color
            } else {
                on_color
            };
            self.put(x, y, color);
            mask >>= 1;
        }
    }

    /// Draws the format info patterns for an encoded number.
    fn draw_format_info_patterns_with_number(&mut self, format_info: u16) {
        let format_info = u32::from(format_info);
        match self.version {
            Version::Micro(_) => {
                self.draw_number(
                    format_info,
                    15,
                    Color::Dark,
                    Color::Light,
                    &FORMAT_INFO_COORDS_MICRO_QR,
                );
            }
            Version::Normal(_) => {
                self.draw_number(
                    format_info,
                    15,
                    Color::Dark,
                    Color::Light,
                    &FORMAT_INFO_COORDS_QR_MAIN,
                );
                self.draw_number(
                    format_info,
                    15,
                    Color::Dark,
                    Color::Light,
                    &FORMAT_INFO_COORDS_QR_SIDE,
                );
                // Dark module.
                self.put(8, -8, Color::Dark);
            }
            Version::RectMicro(..) => {}
        }
    }

    /// Reserves area to put in the format information.
    fn draw_reserved_format_info_patterns(&mut self) {
        self.draw_format_info_patterns_with_number(0);
    }

    /// Draws the version information patterns.
    fn draw_version_info_patterns(&mut self) {
        match self.version {
            Version::Micro(_) | Version::Normal(1..=6) => {}
            Version::Normal(a) => {
                let version_info = VERSION_INFOS[(a - 7).as_usize()];
                self.draw_number(
                    version_info,
                    18,
                    Color::Dark,
                    Color::Light,
                    &VERSION_INFO_COORDS_BL,
                );
                self.draw_number(
                    version_info,
                    18,
                    Color::Dark,
                    Color::Light,
                    &VERSION_INFO_COORDS_TR,
                );
            }
            Version::RectMicro(..) => {
                let index = self.version.rect_micro_index().unwrap();
                let ec_level = usize::from(self.ec_level != EcLevel::M);
                let version_info_left = RMQR_VERSION_INFOS_L[index][ec_level];
                let version_info_right = RMQR_VERSION_INFOS_R[index][ec_level];
                self.draw_number(
                    version_info_left,
                    18,
                    Color::Dark,
                    Color::Light,
                    &RMQR_VERSION_INFO_COORDS_L,
                );
                self.draw_number(
                    version_info_right,
                    18,
                    Color::Dark,
                    Color::Light,
                    &RMQR_VERSION_INFO_COORDS_R,
                );
            }
        }
    }
}

#[cfg(test)]
mod draw_version_info_tests {
    use super::*;

    #[test]
    fn test_draw_number() {
        let mut c = Canvas::new(Version::Micro(1), EcLevel::L);
        c.draw_number(
            0b1010_1101,
            8,
            Color::Dark,
            Color::Light,
            &[(0, 0), (0, -1), (-2, -2), (-2, 0)],
        );
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#????????.?\n",
                "???????????\n",
                "???????????\n",
                "???????????\n",
                "???????????\n",
                "???????????\n",
                "???????????\n",
                "???????????\n",
                "???????????\n",
                "?????????#?\n",
                ".??????????"
            )
        );
    }

    #[test]
    fn test_draw_version_info_1() {
        let mut c = Canvas::new(Version::Normal(1), EcLevel::L);
        c.draw_version_info_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????"
            )
        );
    }

    #[test]
    fn test_draw_version_info_7() {
        let mut c = Canvas::new(Version::Normal(7), EcLevel::L);
        c.draw_version_info_patterns();

        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "??????????????????????????????????..#????????\n",
                "??????????????????????????????????.#.????????\n",
                "??????????????????????????????????.#.????????\n",
                "??????????????????????????????????.##????????\n",
                "??????????????????????????????????###????????\n",
                "??????????????????????????????????...????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "....#.???????????????????????????????????????\n",
                ".####.???????????????????????????????????????\n",
                "#..##.???????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????\n",
                "?????????????????????????????????????????????"
            )
        );
    }

    #[test]
    fn test_draw_reserved_format_info_patterns_qr() {
        let mut c = Canvas::new(Version::Normal(1), EcLevel::L);
        c.draw_reserved_format_info_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "????????.????????????\n",
                "????????.????????????\n",
                "????????.????????????\n",
                "????????.????????????\n",
                "????????.????????????\n",
                "????????.????????????\n",
                "?????????????????????\n",
                "????????.????????????\n",
                "......?..????........\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "?????????????????????\n",
                "????????#????????????\n",
                "????????.????????????\n",
                "????????.????????????\n",
                "????????.????????????\n",
                "????????.????????????\n",
                "????????.????????????\n",
                "????????.????????????\n",
                "????????.????????????"
            )
        );
    }

    #[test]
    fn test_draw_reserved_format_info_patterns_micro_qr() {
        let mut c = Canvas::new(Version::Micro(1), EcLevel::L);
        c.draw_reserved_format_info_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "???????????\n",
                "????????.??\n",
                "????????.??\n",
                "????????.??\n",
                "????????.??\n",
                "????????.??\n",
                "????????.??\n",
                "????????.??\n",
                "?........??\n",
                "???????????\n",
                "???????????"
            )
        );
    }
}

static VERSION_INFO_COORDS_BL: [(i16, i16); 18] = [
    (5, -9),
    (5, -10),
    (5, -11),
    (4, -9),
    (4, -10),
    (4, -11),
    (3, -9),
    (3, -10),
    (3, -11),
    (2, -9),
    (2, -10),
    (2, -11),
    (1, -9),
    (1, -10),
    (1, -11),
    (0, -9),
    (0, -10),
    (0, -11),
];

static VERSION_INFO_COORDS_TR: [(i16, i16); 18] = [
    (-9, 5),
    (-10, 5),
    (-11, 5),
    (-9, 4),
    (-10, 4),
    (-11, 4),
    (-9, 3),
    (-10, 3),
    (-11, 3),
    (-9, 2),
    (-10, 2),
    (-11, 2),
    (-9, 1),
    (-10, 1),
    (-11, 1),
    (-9, 0),
    (-10, 0),
    (-11, 0),
];

static FORMAT_INFO_COORDS_QR_MAIN: [(i16, i16); 15] = [
    (0, 8),
    (1, 8),
    (2, 8),
    (3, 8),
    (4, 8),
    (5, 8),
    (7, 8),
    (8, 8),
    (8, 7),
    (8, 5),
    (8, 4),
    (8, 3),
    (8, 2),
    (8, 1),
    (8, 0),
];

static FORMAT_INFO_COORDS_QR_SIDE: [(i16, i16); 15] = [
    (8, -1),
    (8, -2),
    (8, -3),
    (8, -4),
    (8, -5),
    (8, -6),
    (8, -7),
    (-8, 8),
    (-7, 8),
    (-6, 8),
    (-5, 8),
    (-4, 8),
    (-3, 8),
    (-2, 8),
    (-1, 8),
];

static FORMAT_INFO_COORDS_MICRO_QR: [(i16, i16); 15] = [
    (1, 8),
    (2, 8),
    (3, 8),
    (4, 8),
    (5, 8),
    (6, 8),
    (7, 8),
    (8, 8),
    (8, 7),
    (8, 6),
    (8, 5),
    (8, 4),
    (8, 3),
    (8, 2),
    (8, 1),
];

static VERSION_INFOS: [u32; 34] = [
    0x07C94, 0x085BC, 0x09A99, 0x0A4D3, 0x0BBF6, 0x0C762, 0x0D847, 0x0E60D, 0x0F928, 0x10B78,
    0x1145D, 0x12A17, 0x13532, 0x149A6, 0x15683, 0x168C9, 0x177EC, 0x18EC4, 0x191E1, 0x1AFAB,
    0x1B08E, 0x1CC1A, 0x1D33F, 0x1ED75, 0x1F250, 0x209D5, 0x216F0, 0x228BA, 0x2379F, 0x24B0B,
    0x2542E, 0x26A64, 0x27541, 0x28C69,
];

static RMQR_VERSION_INFO_COORDS_L: [(i16, i16); 18] = [
    (11, 3),
    (11, 2),
    (11, 1),
    (10, 5),
    (10, 4),
    (10, 3),
    (10, 2),
    (10, 1),
    (9, 5),
    (9, 4),
    (9, 3),
    (9, 2),
    (9, 1),
    (8, 5),
    (8, 4),
    (8, 3),
    (8, 2),
    (8, 1),
];

static RMQR_VERSION_INFO_COORDS_R: [(i16, i16); 18] = [
    (-3, -6),
    (-4, -6),
    (-5, -6),
    (-6, -2),
    (-6, -3),
    (-6, -4),
    (-6, -5),
    (-6, -6),
    (-7, -2),
    (-7, -3),
    (-7, -4),
    (-7, -5),
    (-7, -6),
    (-8, -2),
    (-8, -3),
    (-8, -4),
    (-8, -5),
    (-8, -6),
];

/// Version information for finder pattern side. Error correction level (M, H).
static RMQR_VERSION_INFOS_L: [[u32; 2]; 32] = [
    // R7x43
    [0x1FAB2, 0x3F367],
    // R7x59
    [0x1E597, 0x3EC42],
    // R7x77
    [0x1DBDD, 0x3D208],
    // R7x99
    [0x1C4F8, 0x3CD2D],
    // R7x139
    [0x1B86C, 0x3B1B9],
    // R9x43
    [0x1A749, 0x3AE9C],
    // R9x59
    [0x19903, 0x390D6],
    // R9x77
    [0x18626, 0x38FF3],
    // R9x99
    [0x17F0E, 0x376DB],
    // R9x139
    [0x1602B, 0x369FE],
    // R11x27
    [0x15E61, 0x357B4],
    // R11x43
    [0x14144, 0x34891],
    // R11x59
    [0x13DD0, 0x33405],
    // R11x77
    [0x122F5, 0x32B20],
    // R11x99
    [0x11CBF, 0x3156A],
    // R11x139
    [0x1039A, 0x30A4F],
    // R13x27
    [0xF1CA, 0x2F81F],
    // R13x43
    [0xEEEF, 0x2E73A],
    // R13x59
    [0xD0A5, 0x2D970],
    // R13x77
    [0xCF80, 0x2C655],
    // R13x99
    [0xB314, 0x2BAC1],
    // R13x139
    [0xAC31, 0x2A5E4],
    // R15x43
    [0x927B, 0x29BAE],
    // R15x59
    [0x8D5E, 0x2848B],
    // R15x77
    [0x7476, 0x27DA3],
    // R15x99
    [0x6B53, 0x26286],
    // R15x139
    [0x5519, 0x25CCC],
    // R17x43
    [0x4A3C, 0x243E9],
    // R17x59
    [0x36A8, 0x23F7D],
    // R17x77
    [0x298D, 0x22058],
    // R17x99
    [0x17C7, 0x21E12],
    // R17x139
    [0x8E2, 0x20137],
];

/// Version information for finder sub pattern side. Error correction level (M,
/// H).
static RMQR_VERSION_INFOS_R: [[u32; 2]; 32] = [
    // R7x43
    [0x20A7B, 0x3AE],
    // R7x59
    [0x2155E, 0x1C8B],
    // R7x77
    [0x22B14, 0x22C1],
    // R7x99
    [0x23431, 0x3DE4],
    // R7x139
    [0x248A5, 0x4170],
    // R9x43
    [0x25780, 0x5E55],
    // R9x59
    [0x269CA, 0x601F],
    // R9x77
    [0x276EF, 0x7F3A],
    // R9x99
    [0x28FC7, 0x8612],
    // R9x139
    [0x290E2, 0x9937],
    // R11x27
    [0x2AEA8, 0xA77D],
    // R11x43
    [0x2B18D, 0xB858],
    // R11x59
    [0x2CD19, 0xC4CC],
    // R11x77
    [0x2D23C, 0xDBE9],
    // R11x99
    [0x2EC76, 0xE5A3],
    // R11x139
    [0x2F353, 0xFA86],
    // R13x27
    [0x30103, 0x108D6],
    // R13x43
    [0x31E26, 0x117F3],
    // R13x59
    [0x3206C, 0x129B9],
    // R13x77
    [0x33F49, 0x1369C],
    // R13x99
    [0x343DD, 0x14A08],
    // R13x139
    [0x35CF8, 0x1552D],
    // R15x43
    [0x362B2, 0x16B67],
    // R15x59
    [0x37D97, 0x17442],
    // R15x77
    [0x384BF, 0x18D6A],
    // R15x99
    [0x39B9A, 0x1924F],
    // R15x139
    [0x3A5D0, 0x1AC05],
    // R17x43
    [0x3BAF5, 0x1B320],
    // R17x59
    [0x3C661, 0x1CFB4],
    // R17x77
    [0x3D944, 0x1D091],
    // R17x99
    [0x3E70E, 0x1EEDB],
    // R17x139
    [0x3F82B, 0x1F1FE],
];

// All functional patterns before data placement

impl Canvas {
    /// Draws all functional patterns, before data placement.
    ///
    /// All functional patterns (e.g. the finder pattern) _except_ the format
    /// info pattern will be filled in. The format info pattern will be filled
    /// with light modules instead. Data bits can then put in the empty modules.
    /// with [`Canvas::draw_data`].
    pub fn draw_all_functional_patterns(&mut self) {
        self.draw_finder_patterns();
        self.draw_alignment_patterns();
        self.draw_reserved_format_info_patterns();
        self.draw_timing_patterns();
        self.draw_corner_finder_pattern();
        self.draw_alignment_patterns_rmqr();
        self.draw_version_info_patterns();
    }
}

/// Gets whether the module at the given coordinates represents a functional
/// module.
#[must_use]
pub fn is_functional(version: Version, width: i16, x: i16, y: i16) -> bool {
    debug_assert!(width == version.width());

    let x = if x < 0 { x + width } else { x };
    let y = if y < 0 { y + width } else { y };

    match version {
        Version::Micro(_) => x == 0 || y == 0 || (x < 9 && y < 9),
        Version::RectMicro(..) => unimplemented!(),
        Version::Normal(a) => {
            let timing_patterns = x == 6 || y == 6;
            let top_left_finder_pattern = x < 9 && y < 9;
            let bottom_left_finder_pattern = x < 9 && y >= width - 8;
            let top_right_finder_pattern = x >= width - 8 && y < 9;
            let non_alignment_test = timing_patterns
                || top_left_finder_pattern
                || bottom_left_finder_pattern
                || top_right_finder_pattern;
            match a {
                _ if non_alignment_test => true,
                1 => false,
                2..=6 => (width - 7 - x).abs() <= 2 && (width - 7 - y).abs() <= 2,
                _ => {
                    let positions = ALIGNMENT_PATTERN_POSITIONS[(a - 7).as_usize()];
                    let last = positions.len() - 1;
                    for (i, align_x) in positions.iter().enumerate() {
                        for (j, align_y) in positions.iter().enumerate() {
                            if i == 0 && (j == 0 || j == last) || (i == last && j == 0) {
                                continue;
                            }
                            if (*align_x - x).abs() <= 2 && (*align_y - y).abs() <= 2 {
                                return true;
                            }
                        }
                    }
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod all_functional_patterns_tests {
    use super::*;

    #[test]
    fn test_all_functional_patterns_qr() {
        let mut c = Canvas::new(Version::Normal(2), EcLevel::L);
        c.draw_all_functional_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######..????????.#######\n",
                "#.....#..????????.#.....#\n",
                "#.###.#..????????.#.###.#\n",
                "#.###.#..????????.#.###.#\n",
                "#.###.#..????????.#.###.#\n",
                "#.....#..????????.#.....#\n",
                "#######.#.#.#.#.#.#######\n",
                ".........????????........\n",
                "......#..????????........\n",
                "??????.??????????????????\n",
                "??????#??????????????????\n",
                "??????.??????????????????\n",
                "??????#??????????????????\n",
                "??????.??????????????????\n",
                "??????#??????????????????\n",
                "??????.??????????????????\n",
                "??????#?????????#####????\n",
                "........#???????#...#????\n",
                "#######..???????#.#.#????\n",
                "#.....#..???????#...#????\n",
                "#.###.#..???????#####????\n",
                "#.###.#..????????????????\n",
                "#.###.#..????????????????\n",
                "#.....#..????????????????\n",
                "#######..????????????????"
            )
        );
    }

    #[test]
    fn test_all_functional_patterns_micro_qr() {
        let mut c = Canvas::new(Version::Micro(1), EcLevel::L);
        c.draw_all_functional_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######.#.#\n",
                "#.....#..??\n",
                "#.###.#..??\n",
                "#.###.#..??\n",
                "#.###.#..??\n",
                "#.....#..??\n",
                "#######..??\n",
                ".........??\n",
                "#........??\n",
                ".??????????\n",
                "#??????????"
            )
        );
    }

    #[test]
    fn test_is_functional_qr_1() {
        let version = Version::Normal(1);
        assert!(is_functional(version, version.width(), 0, 0));
        assert!(is_functional(version, version.width(), 10, 6));
        assert!(!is_functional(version, version.width(), 10, 5));
        assert!(!is_functional(version, version.width(), 14, 14));
        assert!(is_functional(version, version.width(), 6, 11));
        assert!(!is_functional(version, version.width(), 4, 11));
        assert!(is_functional(version, version.width(), 4, 13));
        assert!(is_functional(version, version.width(), 17, 7));
        assert!(!is_functional(version, version.width(), 17, 17));
    }

    #[test]
    fn test_is_functional_qr_3() {
        let version = Version::Normal(3);
        assert!(is_functional(version, version.width(), 0, 0));
        assert!(!is_functional(version, version.width(), 25, 24));
        assert!(is_functional(version, version.width(), 24, 24));
        assert!(!is_functional(version, version.width(), 9, 25));
        assert!(!is_functional(version, version.width(), 20, 0));
        assert!(is_functional(version, version.width(), 21, 0));
    }

    #[test]
    fn test_is_functional_qr_7() {
        let version = Version::Normal(7);
        assert!(is_functional(version, version.width(), 21, 4));
        assert!(is_functional(version, version.width(), 7, 21));
        assert!(is_functional(version, version.width(), 22, 22));
        assert!(is_functional(version, version.width(), 8, 8));
        assert!(!is_functional(version, version.width(), 19, 5));
        assert!(!is_functional(version, version.width(), 36, 3));
        assert!(!is_functional(version, version.width(), 4, 36));
        assert!(is_functional(version, version.width(), 38, 38));
    }

    #[test]
    fn test_is_functional_micro() {
        let version = Version::Micro(1);
        assert!(is_functional(version, version.width(), 8, 0));
        assert!(is_functional(version, version.width(), 10, 0));
        assert!(!is_functional(version, version.width(), 10, 1));
        assert!(is_functional(version, version.width(), 8, 8));
        assert!(is_functional(version, version.width(), 0, 9));
        assert!(!is_functional(version, version.width(), 1, 9));
    }
}

// Data placement iterator

struct DataModuleIter {
    x: i16,
    y: i16,
    width: i16,
    height: i16,
    timing_pattern_column: i16,
}

impl DataModuleIter {
    const fn new(version: Version) -> Self {
        // In rMQR code, disregarding the bottom and right alignment patterns works
        // well.
        let (width, height) = if let Version::RectMicro(..) = version {
            (version.width() - 1, version.height() - 1)
        } else {
            (version.width(), version.height())
        };
        let timing_pattern_column = if let Version::Normal(_) = version {
            6
        } else {
            0
        };

        let (x, y) = (width - 1, height - 1);
        Self {
            x,
            y,
            width,
            height,
            timing_pattern_column,
        }
    }
}

impl Iterator for DataModuleIter {
    type Item = (i16, i16);

    fn next(&mut self) -> Option<Self::Item> {
        let adjusted_ref_col = if self.x <= self.timing_pattern_column {
            self.x + 1
        } else {
            self.x
        };
        if adjusted_ref_col <= 0 {
            return None;
        }

        let res = (self.x, self.y);
        let column_type = (self.width - adjusted_ref_col) % 4;

        match column_type {
            2 if self.y > 0 => {
                self.y -= 1;
                self.x += 1;
            }
            0 if self.y < self.height - 1 => {
                self.y += 1;
                self.x += 1;
            }
            0 | 2 if self.x == self.timing_pattern_column + 1 => {
                self.x -= 2;
            }
            _ => {
                self.x -= 1;
            }
        }

        Some(res)
    }
}

#[cfg(test)]
mod data_iter_tests {
    use super::*;

    #[test]
    fn test_qr() {
        let res = DataModuleIter::new(Version::Normal(1)).collect::<Vec<(i16, i16)>>();
        assert_eq!(
            res,
            [
                (20, 20),
                (19, 20),
                (20, 19),
                (19, 19),
                (20, 18),
                (19, 18),
                (20, 17),
                (19, 17),
                (20, 16),
                (19, 16),
                (20, 15),
                (19, 15),
                (20, 14),
                (19, 14),
                (20, 13),
                (19, 13),
                (20, 12),
                (19, 12),
                (20, 11),
                (19, 11),
                (20, 10),
                (19, 10),
                (20, 9),
                (19, 9),
                (20, 8),
                (19, 8),
                (20, 7),
                (19, 7),
                (20, 6),
                (19, 6),
                (20, 5),
                (19, 5),
                (20, 4),
                (19, 4),
                (20, 3),
                (19, 3),
                (20, 2),
                (19, 2),
                (20, 1),
                (19, 1),
                (20, 0),
                (19, 0),
                (18, 0),
                (17, 0),
                (18, 1),
                (17, 1),
                (18, 2),
                (17, 2),
                (18, 3),
                (17, 3),
                (18, 4),
                (17, 4),
                (18, 5),
                (17, 5),
                (18, 6),
                (17, 6),
                (18, 7),
                (17, 7),
                (18, 8),
                (17, 8),
                (18, 9),
                (17, 9),
                (18, 10),
                (17, 10),
                (18, 11),
                (17, 11),
                (18, 12),
                (17, 12),
                (18, 13),
                (17, 13),
                (18, 14),
                (17, 14),
                (18, 15),
                (17, 15),
                (18, 16),
                (17, 16),
                (18, 17),
                (17, 17),
                (18, 18),
                (17, 18),
                (18, 19),
                (17, 19),
                (18, 20),
                (17, 20),
                (16, 20),
                (15, 20),
                (16, 19),
                (15, 19),
                (16, 18),
                (15, 18),
                (16, 17),
                (15, 17),
                (16, 16),
                (15, 16),
                (16, 15),
                (15, 15),
                (16, 14),
                (15, 14),
                (16, 13),
                (15, 13),
                (16, 12),
                (15, 12),
                (16, 11),
                (15, 11),
                (16, 10),
                (15, 10),
                (16, 9),
                (15, 9),
                (16, 8),
                (15, 8),
                (16, 7),
                (15, 7),
                (16, 6),
                (15, 6),
                (16, 5),
                (15, 5),
                (16, 4),
                (15, 4),
                (16, 3),
                (15, 3),
                (16, 2),
                (15, 2),
                (16, 1),
                (15, 1),
                (16, 0),
                (15, 0),
                (14, 0),
                (13, 0),
                (14, 1),
                (13, 1),
                (14, 2),
                (13, 2),
                (14, 3),
                (13, 3),
                (14, 4),
                (13, 4),
                (14, 5),
                (13, 5),
                (14, 6),
                (13, 6),
                (14, 7),
                (13, 7),
                (14, 8),
                (13, 8),
                (14, 9),
                (13, 9),
                (14, 10),
                (13, 10),
                (14, 11),
                (13, 11),
                (14, 12),
                (13, 12),
                (14, 13),
                (13, 13),
                (14, 14),
                (13, 14),
                (14, 15),
                (13, 15),
                (14, 16),
                (13, 16),
                (14, 17),
                (13, 17),
                (14, 18),
                (13, 18),
                (14, 19),
                (13, 19),
                (14, 20),
                (13, 20),
                (12, 20),
                (11, 20),
                (12, 19),
                (11, 19),
                (12, 18),
                (11, 18),
                (12, 17),
                (11, 17),
                (12, 16),
                (11, 16),
                (12, 15),
                (11, 15),
                (12, 14),
                (11, 14),
                (12, 13),
                (11, 13),
                (12, 12),
                (11, 12),
                (12, 11),
                (11, 11),
                (12, 10),
                (11, 10),
                (12, 9),
                (11, 9),
                (12, 8),
                (11, 8),
                (12, 7),
                (11, 7),
                (12, 6),
                (11, 6),
                (12, 5),
                (11, 5),
                (12, 4),
                (11, 4),
                (12, 3),
                (11, 3),
                (12, 2),
                (11, 2),
                (12, 1),
                (11, 1),
                (12, 0),
                (11, 0),
                (10, 0),
                (9, 0),
                (10, 1),
                (9, 1),
                (10, 2),
                (9, 2),
                (10, 3),
                (9, 3),
                (10, 4),
                (9, 4),
                (10, 5),
                (9, 5),
                (10, 6),
                (9, 6),
                (10, 7),
                (9, 7),
                (10, 8),
                (9, 8),
                (10, 9),
                (9, 9),
                (10, 10),
                (9, 10),
                (10, 11),
                (9, 11),
                (10, 12),
                (9, 12),
                (10, 13),
                (9, 13),
                (10, 14),
                (9, 14),
                (10, 15),
                (9, 15),
                (10, 16),
                (9, 16),
                (10, 17),
                (9, 17),
                (10, 18),
                (9, 18),
                (10, 19),
                (9, 19),
                (10, 20),
                (9, 20),
                (8, 20),
                (7, 20),
                (8, 19),
                (7, 19),
                (8, 18),
                (7, 18),
                (8, 17),
                (7, 17),
                (8, 16),
                (7, 16),
                (8, 15),
                (7, 15),
                (8, 14),
                (7, 14),
                (8, 13),
                (7, 13),
                (8, 12),
                (7, 12),
                (8, 11),
                (7, 11),
                (8, 10),
                (7, 10),
                (8, 9),
                (7, 9),
                (8, 8),
                (7, 8),
                (8, 7),
                (7, 7),
                (8, 6),
                (7, 6),
                (8, 5),
                (7, 5),
                (8, 4),
                (7, 4),
                (8, 3),
                (7, 3),
                (8, 2),
                (7, 2),
                (8, 1),
                (7, 1),
                (8, 0),
                (7, 0),
                (5, 0),
                (4, 0),
                (5, 1),
                (4, 1),
                (5, 2),
                (4, 2),
                (5, 3),
                (4, 3),
                (5, 4),
                (4, 4),
                (5, 5),
                (4, 5),
                (5, 6),
                (4, 6),
                (5, 7),
                (4, 7),
                (5, 8),
                (4, 8),
                (5, 9),
                (4, 9),
                (5, 10),
                (4, 10),
                (5, 11),
                (4, 11),
                (5, 12),
                (4, 12),
                (5, 13),
                (4, 13),
                (5, 14),
                (4, 14),
                (5, 15),
                (4, 15),
                (5, 16),
                (4, 16),
                (5, 17),
                (4, 17),
                (5, 18),
                (4, 18),
                (5, 19),
                (4, 19),
                (5, 20),
                (4, 20),
                (3, 20),
                (2, 20),
                (3, 19),
                (2, 19),
                (3, 18),
                (2, 18),
                (3, 17),
                (2, 17),
                (3, 16),
                (2, 16),
                (3, 15),
                (2, 15),
                (3, 14),
                (2, 14),
                (3, 13),
                (2, 13),
                (3, 12),
                (2, 12),
                (3, 11),
                (2, 11),
                (3, 10),
                (2, 10),
                (3, 9),
                (2, 9),
                (3, 8),
                (2, 8),
                (3, 7),
                (2, 7),
                (3, 6),
                (2, 6),
                (3, 5),
                (2, 5),
                (3, 4),
                (2, 4),
                (3, 3),
                (2, 3),
                (3, 2),
                (2, 2),
                (3, 1),
                (2, 1),
                (3, 0),
                (2, 0),
                (1, 0),
                (0, 0),
                (1, 1),
                (0, 1),
                (1, 2),
                (0, 2),
                (1, 3),
                (0, 3),
                (1, 4),
                (0, 4),
                (1, 5),
                (0, 5),
                (1, 6),
                (0, 6),
                (1, 7),
                (0, 7),
                (1, 8),
                (0, 8),
                (1, 9),
                (0, 9),
                (1, 10),
                (0, 10),
                (1, 11),
                (0, 11),
                (1, 12),
                (0, 12),
                (1, 13),
                (0, 13),
                (1, 14),
                (0, 14),
                (1, 15),
                (0, 15),
                (1, 16),
                (0, 16),
                (1, 17),
                (0, 17),
                (1, 18),
                (0, 18),
                (1, 19),
                (0, 19),
                (1, 20),
                (0, 20),
            ]
        );
    }

    #[test]
    fn test_micro_qr() {
        let res = DataModuleIter::new(Version::Micro(1)).collect::<Vec<(i16, i16)>>();
        assert_eq!(
            res,
            [
                (10, 10),
                (9, 10),
                (10, 9),
                (9, 9),
                (10, 8),
                (9, 8),
                (10, 7),
                (9, 7),
                (10, 6),
                (9, 6),
                (10, 5),
                (9, 5),
                (10, 4),
                (9, 4),
                (10, 3),
                (9, 3),
                (10, 2),
                (9, 2),
                (10, 1),
                (9, 1),
                (10, 0),
                (9, 0),
                (8, 0),
                (7, 0),
                (8, 1),
                (7, 1),
                (8, 2),
                (7, 2),
                (8, 3),
                (7, 3),
                (8, 4),
                (7, 4),
                (8, 5),
                (7, 5),
                (8, 6),
                (7, 6),
                (8, 7),
                (7, 7),
                (8, 8),
                (7, 8),
                (8, 9),
                (7, 9),
                (8, 10),
                (7, 10),
                (6, 10),
                (5, 10),
                (6, 9),
                (5, 9),
                (6, 8),
                (5, 8),
                (6, 7),
                (5, 7),
                (6, 6),
                (5, 6),
                (6, 5),
                (5, 5),
                (6, 4),
                (5, 4),
                (6, 3),
                (5, 3),
                (6, 2),
                (5, 2),
                (6, 1),
                (5, 1),
                (6, 0),
                (5, 0),
                (4, 0),
                (3, 0),
                (4, 1),
                (3, 1),
                (4, 2),
                (3, 2),
                (4, 3),
                (3, 3),
                (4, 4),
                (3, 4),
                (4, 5),
                (3, 5),
                (4, 6),
                (3, 6),
                (4, 7),
                (3, 7),
                (4, 8),
                (3, 8),
                (4, 9),
                (3, 9),
                (4, 10),
                (3, 10),
                (2, 10),
                (1, 10),
                (2, 9),
                (1, 9),
                (2, 8),
                (1, 8),
                (2, 7),
                (1, 7),
                (2, 6),
                (1, 6),
                (2, 5),
                (1, 5),
                (2, 4),
                (1, 4),
                (2, 3),
                (1, 3),
                (2, 2),
                (1, 2),
                (2, 1),
                (1, 1),
                (2, 0),
                (1, 0),
            ]
        );
    }

    #[test]
    fn test_micro_qr_2() {
        let res = DataModuleIter::new(Version::Micro(2)).collect::<Vec<(i16, i16)>>();
        assert_eq!(
            res,
            [
                (12, 12),
                (11, 12),
                (12, 11),
                (11, 11),
                (12, 10),
                (11, 10),
                (12, 9),
                (11, 9),
                (12, 8),
                (11, 8),
                (12, 7),
                (11, 7),
                (12, 6),
                (11, 6),
                (12, 5),
                (11, 5),
                (12, 4),
                (11, 4),
                (12, 3),
                (11, 3),
                (12, 2),
                (11, 2),
                (12, 1),
                (11, 1),
                (12, 0),
                (11, 0),
                (10, 0),
                (9, 0),
                (10, 1),
                (9, 1),
                (10, 2),
                (9, 2),
                (10, 3),
                (9, 3),
                (10, 4),
                (9, 4),
                (10, 5),
                (9, 5),
                (10, 6),
                (9, 6),
                (10, 7),
                (9, 7),
                (10, 8),
                (9, 8),
                (10, 9),
                (9, 9),
                (10, 10),
                (9, 10),
                (10, 11),
                (9, 11),
                (10, 12),
                (9, 12),
                (8, 12),
                (7, 12),
                (8, 11),
                (7, 11),
                (8, 10),
                (7, 10),
                (8, 9),
                (7, 9),
                (8, 8),
                (7, 8),
                (8, 7),
                (7, 7),
                (8, 6),
                (7, 6),
                (8, 5),
                (7, 5),
                (8, 4),
                (7, 4),
                (8, 3),
                (7, 3),
                (8, 2),
                (7, 2),
                (8, 1),
                (7, 1),
                (8, 0),
                (7, 0),
                (6, 0),
                (5, 0),
                (6, 1),
                (5, 1),
                (6, 2),
                (5, 2),
                (6, 3),
                (5, 3),
                (6, 4),
                (5, 4),
                (6, 5),
                (5, 5),
                (6, 6),
                (5, 6),
                (6, 7),
                (5, 7),
                (6, 8),
                (5, 8),
                (6, 9),
                (5, 9),
                (6, 10),
                (5, 10),
                (6, 11),
                (5, 11),
                (6, 12),
                (5, 12),
                (4, 12),
                (3, 12),
                (4, 11),
                (3, 11),
                (4, 10),
                (3, 10),
                (4, 9),
                (3, 9),
                (4, 8),
                (3, 8),
                (4, 7),
                (3, 7),
                (4, 6),
                (3, 6),
                (4, 5),
                (3, 5),
                (4, 4),
                (3, 4),
                (4, 3),
                (3, 3),
                (4, 2),
                (3, 2),
                (4, 1),
                (3, 1),
                (4, 0),
                (3, 0),
                (2, 0),
                (1, 0),
                (2, 1),
                (1, 1),
                (2, 2),
                (1, 2),
                (2, 3),
                (1, 3),
                (2, 4),
                (1, 4),
                (2, 5),
                (1, 5),
                (2, 6),
                (1, 6),
                (2, 7),
                (1, 7),
                (2, 8),
                (1, 8),
                (2, 9),
                (1, 9),
                (2, 10),
                (1, 10),
                (2, 11),
                (1, 11),
                (2, 12),
                (1, 12),
            ]
        );
    }
}

// Data placement

impl Canvas {
    fn draw_codewords<I>(&mut self, codewords: &[u8], is_half_codeword_at_end: bool, coords: &mut I)
    where
        I: Iterator<Item = (i16, i16)>,
    {
        let length = codewords.len();
        let last_word = if is_half_codeword_at_end {
            length - 1
        } else {
            length
        };
        for (i, b) in codewords.iter().enumerate() {
            let bits_end = if i == last_word { 4 } else { 0 };
            'outside: for j in (bits_end..=7).rev() {
                let color = if (*b & (1 << j)) == 0 {
                    Color::Light
                } else {
                    Color::Dark
                };
                for (x, y) in coords.by_ref() {
                    let r = self.get_mut(x, y);
                    if *r == Module::Empty {
                        *r = Module::Unmasked(color);
                        continue 'outside;
                    }
                }
                return;
            }
        }
    }

    /// Draws the encoded data and error correction codes to the empty modules.
    pub fn draw_data(&mut self, data: &[u8], ec: &[u8]) {
        let is_half_codeword_at_end = matches!(
            (self.version, self.ec_level),
            (Version::Micro(1 | 3), EcLevel::L) | (Version::Micro(3), EcLevel::M)
        );
        let mut coords = DataModuleIter::new(self.version);
        self.draw_codewords(data, is_half_codeword_at_end, &mut coords);
        self.draw_codewords(ec, false, &mut coords);
    }
}

#[cfg(test)]
mod draw_codewords_tests {
    use super::*;

    #[test]
    fn test_micro_qr_1() {
        let mut c = Canvas::new(Version::Micro(1), EcLevel::L);
        c.draw_all_functional_patterns();
        c.draw_data(b"\x6E\x5D\xE2", b"\x2B\x63");
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######.#.#\n",
                "#.....#..-*\n",
                "#.###.#..**\n",
                "#.###.#..*-\n",
                "#.###.#..**\n",
                "#.....#..*-\n",
                "#######..*-\n",
                ".........-*\n",
                "#........**\n",
                ".***-**---*\n",
                "#---*-*-**-"
            )
        );
    }

    #[test]
    fn test_qr_2() {
        let mut c = Canvas::new(Version::Normal(2), EcLevel::L);
        c.draw_all_functional_patterns();
        c.draw_data(
            &[
                0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49,
                0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92,
                0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24,
                0x92, 0x49, 0x24,
            ],
            b"",
        );
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######..--*---*-.#######\n",
                "#.....#..-*-*-*-*.#.....#\n",
                "#.###.#..*---*---.#.###.#\n",
                "#.###.#..--*---*-.#.###.#\n",
                "#.###.#..-*-*-*-*.#.###.#\n",
                "#.....#..*---*---.#.....#\n",
                "#######.#.#.#.#.#.#######\n",
                ".........--*---*-........\n",
                "......#..-*-*-*-*........\n",
                "--*-*-.-**---*---*--**--*\n",
                "-*-*--#----*---*---------\n",
                "*----*.*--*-*-*-*-**--**-\n",
                "--*-*-#-**---*---*--**--*\n",
                "-*-*--.----*---*---------\n",
                "*----*#*--*-*-*-*-**--**-\n",
                "--*-*-.-**---*---*--**--*\n",
                "-*-*--#----*---*#####----\n",
                "........#-*-*-*-#...#-**-\n",
                "#######..*---*--#.#.#*--*\n",
                "#.....#..--*---*#...#----\n",
                "#.###.#..-*-*-*-#####-**-\n",
                "#.###.#..*---*--*----*--*\n",
                "#.###.#..--*------**-----\n",
                "#.....#..-*-*-**-*--*-**-\n",
                "#######..*---*--*----*--*"
            )
        );
    }

    #[test]
    fn test_rmqr() {
        let mut c = Canvas::new(Version::RectMicro(7, 77), EcLevel::M);
        c.draw_all_functional_patterns();
        c.draw_data(
            &[
                0x71, 0x68, 0x74, 0x74, 0x70, 0x73, 0x3A, 0x2F, 0x2F, 0x6F, 0x75, 0x64, 0x6F, 0x6E,
                0x2E, 0x78, 0x79, 0x7A, 0x00, 0xEC, 0xFF, 0x6B, 0xC6, 0xCB, 0x02, 0x06, 0xA5, 0xFE,
                0x36, 0x6E, 0x55, 0xFF,
            ],
            b"",
        );
        assert_eq!(
            c.to_debug_str_mask_same(),
            concat!(
                "\n",
                "#######.#.#.#.#.#.#.#.#.###.#.#.#.#.#.#.#.#.#.#.#.###.#.#.#.#.#.#.#.#.#.#.###\n",
                "#.....#.#..#######.##.#.#.#..##.####.#.####.#######.#.##..#..###..........#.#\n",
                "#.###.#..#####.#..#...#####.##..##....##.#.##..##.####.##.##....###.#..######\n",
                "#.###.#.###.##.###...#..#.....######..#...##.##.#.#.#####.#....#.#####..#...#\n",
                "#.###.#.##.??#.##.####..###..####..#..#.#..###..#.#####.##.###.#.#.##.###.#.#\n",
                "#.....#.###???..######..#.#.##.#.###...###...##..##.#.#..##..###.....##.#...#\n",
                "#######.#.#.#.#.#.#.#.#.###.#.#.#.#.#.#.#.#.#.#.#.###.#.#.#.#.#.#.#.#.#.#####"
            )
        );
    }
}

// Masking

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

#[cfg(test)]
mod mask_tests {
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

// Penalty score

impl Canvas {
    /// Computes the penalty score for having too many adjacent modules with the
    /// same color.
    ///
    /// Every 5+N adjacent modules in the same column/row having the same color
    /// will contribute 3+N points.
    fn compute_adjacent_penalty_score(&self, is_horizontal: bool) -> u16 {
        let mut total_score = 0;

        for i in 0..self.width {
            let map_fn = |j| {
                if is_horizontal {
                    self.get(j, i)
                } else {
                    self.get(i, j)
                }
            };

            let colors = (0..self.width).map(map_fn).chain(iter::once(Module::Empty));
            let mut last_color = Module::Empty;
            let mut consecutive_len = 1_u16;

            for color in colors {
                if color == last_color {
                    consecutive_len += 1;
                } else {
                    last_color = color;
                    if consecutive_len >= 5 {
                        total_score += consecutive_len - 2;
                    }
                    consecutive_len = 1;
                }
            }
        }

        total_score
    }

    /// Computes the penalty score for having too many rectangles with the same
    /// color.
    ///
    /// Every 2×2 blocks (with overlapping counted) having the same color will
    /// contribute 3 points.
    fn compute_block_penalty_score(&self) -> u16 {
        let mut total_score = 0;

        for i in 0..self.width - 1 {
            for j in 0..self.width - 1 {
                let this = self.get(i, j);
                let right = self.get(i + 1, j);
                let bottom = self.get(i, j + 1);
                let bottom_right = self.get(i + 1, j + 1);
                if this == right && right == bottom && bottom == bottom_right {
                    total_score += 3;
                }
            }
        }

        total_score
    }

    /// Computes the penalty score for having a pattern similar to the finder
    /// pattern in the wrong place.
    ///
    /// Every pattern that looks like `#.###.#....` in any orientation will add
    /// 40 points.
    fn compute_finder_penalty_score(&self, is_horizontal: bool) -> u16 {
        static PATTERN: [Color; 7] = [
            Color::Dark,
            Color::Light,
            Color::Dark,
            Color::Dark,
            Color::Dark,
            Color::Light,
            Color::Dark,
        ];

        let mut total_score = 0;

        for i in 0..self.width {
            for j in 0..self.width - 6 {
                // TODO a ref to a closure should be enough?
                let get: Box<dyn Fn(i16) -> Color> = if is_horizontal {
                    Box::new(|k| self.get(k, i).into())
                } else {
                    Box::new(|k| self.get(i, k).into())
                };

                if (j..(j + 7)).map(&get).ne(PATTERN.iter().copied()) {
                    continue;
                }

                let check = |k| 0 <= k && k < self.width && get(k) != Color::Light;
                if !((j - 4)..j).any(&check) || !((j + 7)..(j + 11)).any(&check) {
                    total_score += 40;
                }
            }
        }

        total_score - 360
    }

    /// Computes the penalty score for having an unbalanced dark/light ratio.
    ///
    /// The score is given linearly by the deviation from a 50% ratio of dark
    /// modules. The highest possible score is 100.
    ///
    /// <div class="warning">
    ///
    /// Note that this algorithm differs slightly from the standard we do not
    /// round the result every 5%, but the difference should be negligible and
    /// should not affect which mask is chosen.
    ///
    /// </div>
    fn compute_balance_penalty_score(&self) -> u16 {
        let dark_modules = self.modules.iter().filter(|m| m.is_dark()).count();
        let total_modules = self.modules.len();
        let ratio = dark_modules * 200 / total_modules;
        ratio.abs_diff(100).as_u16()
    }

    /// Computes the penalty score for having too many light modules on the
    /// sides.
    ///
    /// This penalty score is exclusive to Micro QR code.
    ///
    /// <div class="warning">
    ///
    /// Note that the standard gives the formula for *efficiency* score, which
    /// has the inverse meaning of this method, but it is very easy to convert
    /// between the two (this score is (16×width − standard-score)).
    ///
    /// </div>
    fn compute_light_side_penalty_score(&self) -> u16 {
        let h = (1..self.width)
            .filter(|j| !self.get(*j, -1).is_dark())
            .count();
        let v = (1..self.width)
            .filter(|j| !self.get(-1, *j).is_dark())
            .count();

        (h + v + 15 * cmp::max(h, v)).as_u16()
    }

    /// Computes the total penalty scores. A QR code having higher points is
    /// less desirable.
    fn compute_total_penalty_scores(&self) -> u16 {
        match self.version {
            Version::Normal(_) => {
                let s1_a = self.compute_adjacent_penalty_score(true);
                let s1_b = self.compute_adjacent_penalty_score(false);
                let s2 = self.compute_block_penalty_score();
                let s3_a = self.compute_finder_penalty_score(true);
                let s3_b = self.compute_finder_penalty_score(false);
                let s4 = self.compute_balance_penalty_score();
                s1_a + s1_b + s2 + s3_a + s3_b + s4
            }
            Version::Micro(_) => self.compute_light_side_penalty_score(),
            Version::RectMicro(..) => 0,
        }
    }
}

#[cfg(test)]
mod penalty_tests {
    use super::*;

    fn create_test_canvas() -> Canvas {
        let mut c = Canvas::new(Version::Normal(1), EcLevel::Q);
        c.draw_all_functional_patterns();
        c.draw_data(
            b"\x20\x5B\x0B\x78\xD1\x72\xDC\x4D\x43\x40\xEC\x11\x00",
            b"\xA8\x48\x16\x52\xD9\x36\x9C\x00\x2E\x0F\xB4\x7A\x10",
        );
        c.apply_mask(MaskPattern::Checkerboard);
        c
    }

    #[test]
    fn check_penalty_canvas() {
        let c = create_test_canvas();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######.##....#######\n",
                "#.....#.#..#..#.....#\n",
                "#.###.#.#..##.#.###.#\n",
                "#.###.#.#.....#.###.#\n",
                "#.###.#.#.#...#.###.#\n",
                "#.....#...#...#.....#\n",
                "#######.#.#.#.#######\n",
                "........#............\n",
                ".##.#.##....#.#.#####\n",
                ".#......####....#...#\n",
                "..##.###.##...#.##...\n",
                ".##.##.#..##.#.#.###.\n",
                "#...#.#.#.###.###.#.#\n",
                "........##.#..#...#.#\n",
                "#######.#.#....#.##..\n",
                "#.....#..#.##.##.#...\n",
                "#.###.#.#.#...#######\n",
                "#.###.#..#.#.#.#...#.\n",
                "#.###.#.#...####.#..#\n",
                "#.....#.#.##.#...#.##\n",
                "#######.....####....#"
            )
        );
    }

    #[test]
    fn test_penalty_score_adjacent() {
        let c = create_test_canvas();
        assert_eq!(c.compute_adjacent_penalty_score(true), 88);
        assert_eq!(c.compute_adjacent_penalty_score(false), 92);
    }

    #[test]
    fn test_penalty_score_block() {
        let c = create_test_canvas();
        assert_eq!(c.compute_block_penalty_score(), 90);
    }

    #[test]
    fn test_penalty_score_finder() {
        let c = create_test_canvas();
        assert_eq!(c.compute_finder_penalty_score(true), 0);
        assert_eq!(c.compute_finder_penalty_score(false), 40);
    }

    #[test]
    fn test_penalty_score_balance() {
        let c = create_test_canvas();
        assert_eq!(c.compute_balance_penalty_score(), 2);
    }

    #[test]
    fn test_penalty_score_light_sides() {
        static HORIZONTAL_SIDE: [Color; 17] = [
            Color::Dark,
            Color::Light,
            Color::Light,
            Color::Dark,
            Color::Dark,
            Color::Dark,
            Color::Light,
            Color::Light,
            Color::Dark,
            Color::Light,
            Color::Dark,
            Color::Light,
            Color::Light,
            Color::Dark,
            Color::Light,
            Color::Light,
            Color::Light,
        ];
        static VERTICAL_SIDE: [Color; 17] = [
            Color::Dark,
            Color::Dark,
            Color::Dark,
            Color::Light,
            Color::Light,
            Color::Dark,
            Color::Dark,
            Color::Light,
            Color::Dark,
            Color::Light,
            Color::Dark,
            Color::Light,
            Color::Dark,
            Color::Light,
            Color::Light,
            Color::Dark,
            Color::Light,
        ];

        let mut c = Canvas::new(Version::Micro(4), EcLevel::Q);
        for i in 0_i16..17 {
            c.put(i, -1, HORIZONTAL_SIDE[i as usize]);
            c.put(-1, i, VERTICAL_SIDE[i as usize]);
        }

        assert_eq!(c.compute_light_side_penalty_score(), 168);
    }
}

// Select mask with lowest penalty score

static ALL_PATTERNS_QR: [MaskPattern; 8] = [
    MaskPattern::Checkerboard,
    MaskPattern::HorizontalLines,
    MaskPattern::VerticalLines,
    MaskPattern::DiagonalLines,
    MaskPattern::LargeCheckerboard,
    MaskPattern::Fields,
    MaskPattern::Diamonds,
    MaskPattern::Meadow,
];

static ALL_PATTERNS_MICRO_QR: [MaskPattern; 4] = [
    MaskPattern::HorizontalLines,
    MaskPattern::LargeCheckerboard,
    MaskPattern::Diamonds,
    MaskPattern::Meadow,
];

static ALL_PATTERNS_RMQR: [MaskPattern; 1] = [MaskPattern::LargeCheckerboard];

impl Canvas {
    #[expect(clippy::missing_panics_doc)]
    /// Constructs a new canvas and apply the best masking that gives the lowest
    /// penalty score.
    #[must_use]
    pub fn apply_best_mask(&self) -> Self {
        match self.version {
            Version::Normal(_) => ALL_PATTERNS_QR.iter(),
            Version::Micro(_) => ALL_PATTERNS_MICRO_QR.iter(),
            Version::RectMicro(..) => ALL_PATTERNS_RMQR.iter(),
        }
        .map(|ptn| {
            let mut c = self.clone();
            c.apply_mask(*ptn);
            c
        })
        .min_by_key(Self::compute_total_penalty_scores)
        .unwrap()
    }

    /// Converts the modules into a vector of colors.
    pub fn into_colors(self) -> Vec<Color> {
        self.modules.into_iter().map(Color::from).collect()
    }
}
