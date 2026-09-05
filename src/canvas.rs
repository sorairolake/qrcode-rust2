// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2018 Ignas Anikevicius
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The `canvas` module puts raw bits into the QR code canvas.
//!
//! # Examples
//!
//! ```
//! use qrcode2::{
//!     EcLevel, NormalVersion, Version,
//!     canvas::{Canvas, MaskPattern},
//! };
//!
//! let mut c = Canvas::new(Version::Normal(NormalVersion::V1), EcLevel::L);
//! c.draw_all_functional_patterns();
//! c.draw_data(b"data_here", b"ec_code_here");
//! c.apply_mask(MaskPattern::Checkerboard);
//! let colors = c.into_colors();
//! ```

mod alignment;
mod corner_finder;
mod data_module_iter;
mod finder;
mod functional;
mod info;
mod mask;
mod module;
mod optimize;
mod penalty;
mod placement;
mod timing;

#[cfg(test)]
use alloc::string::String;
use alloc::{vec, vec::Vec};

pub use self::{functional::is_functional, mask::MaskPattern, module::Module};
use crate::types::{Color, EcLevel, Version};

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
    #[expect(clippy::missing_panics_doc)]
    /// Constructs a new canvas big enough for a QR code of the given version.
    #[must_use]
    pub fn new(version: Version, ec_level: EcLevel) -> Self {
        let (width, height) = (version.width(), version.height());
        let modules = vec![Module::Empty; (height * width).try_into().unwrap()];
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
        let mut res = String::with_capacity((width * (width + 1)).try_into().unwrap());
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
        let mut res = String::with_capacity((width * (width + 1)).try_into().unwrap());
        for y in 0..self.height {
            res.push('\n');
            for x in 0..width {
                res.push(match self.get(x, y) {
                    Module::Empty => '?',
                    Module::Masked(Color::Light) | Module::Unmasked(Color::Light) => '.',
                    Module::Masked(Color::Dark) | Module::Unmasked(Color::Dark) => '#',
                });
            }
        }
        res
    }

    fn coords_to_index(&self, x: i16, y: i16) -> usize {
        let x = if x < 0 { x + self.width } else { x };
        let x = usize::try_from(x).unwrap();
        let y = if y < 0 { y + self.height } else { y };
        let y = usize::try_from(y).unwrap();
        y * usize::try_from(self.width).unwrap() + x
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
mod tests {
    use super::*;
    use crate::types::NormalVersion;

    #[test]
    fn index() {
        let mut c = Canvas::new(Version::Normal(NormalVersion::V1), EcLevel::L);

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
    fn debug_str() {
        let mut c = Canvas::new(Version::Normal(NormalVersion::V1), EcLevel::L);

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
