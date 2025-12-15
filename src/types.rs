// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2018 Ethan Pailes
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The `types` module contains types associated with the functional elements of
//! a QR code.

use core::{cmp::Ordering, error::Error, fmt, ops::Not};

use crate::cast::As;

// `QrResult`

/// `QrError` encodes the error encountered when generating a QR code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QrError {
    /// The data is too long to encode into a QR code for the given version.
    DataTooLong,

    /// The provided version / error correction level combination is invalid.
    InvalidVersion,

    /// Some characters in the data cannot be supported by the provided QR code
    /// version.
    UnsupportedCharacterSet,

    /// The provided ECI designator is invalid. A valid designator should be
    /// between 0 and 999,999.
    InvalidEciDesignator,

    /// A character not belonging to the character set is found.
    InvalidCharacter,
}

impl fmt::Display for QrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataTooLong => write!(f, "data too long"),
            Self::InvalidVersion => write!(f, "invalid version"),
            Self::UnsupportedCharacterSet => write!(f, "unsupported character set"),
            Self::InvalidEciDesignator => write!(f, "invalid ECI designator"),
            Self::InvalidCharacter => write!(f, "invalid character"),
        }
    }
}

impl Error for QrError {}

/// `QrResult` is a convenient alias for a QR code generation result.
pub type QrResult<T> = Result<T, QrError>;

// Color

/// The color of a module.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    /// The module is light colored.
    Light,

    /// The module is dark colored.
    Dark,
}

impl Color {
    /// Selects a value according to color of the module.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::Color;
    /// #
    /// assert_eq!(Color::Light.select(1, 0), 0);
    /// assert_eq!(Color::Dark.select("black", "white"), "black");
    /// ```
    pub fn select<T>(self, dark: T, light: T) -> T {
        match self {
            Self::Light => light,
            Self::Dark => dark,
        }
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

// Error correction level

/// The error correction level. It allows the original information be recovered
/// even if parts of the code is damaged.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum EcLevel {
    /// Low error correction. Allows up to 7% of wrong blocks.
    L = 0,

    /// Medium error correction. Allows up to 15% of wrong blocks.
    #[default]
    M = 1,

    /// "Quartile" error correction. Allows up to 25% of wrong blocks.
    Q = 2,

    /// High error correction. Allows up to 30% of wrong blocks.
    H = 3,
}

#[cfg(test)]
mod ec_level_tests {
    use super::*;

    #[test]
    fn test_default() {
        assert_eq!(EcLevel::default(), EcLevel::M);
    }
}

// Version

/// In QR code terminology, `Version` means the size of the generated image.
/// Larger version means the size of code is larger, and therefore can carry
/// more information.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Version {
    /// A normal QR code version. The parameter should be between 1 and 40. The
    /// smallest version is `Version::Normal(1)` of size 21×21, and the largest
    /// is `Version::Normal(40)` of size 177×177.
    Normal(i16),

    /// A Micro QR code version. The parameter should be between 1 and 4. The
    /// smallest version is `Version::Micro(1)` of size 11×11, and the largest
    /// is `Version::Micro(4)` of size 17×17.
    Micro(i16),

    /// A rMQR code version. The first parameter represents the height and
    /// should be 7, 9, 11, 13, 15, or 17. The second parameter represents the
    /// width and should be 27, 43, 59, 77, 99, or 139. 27 can only be used with
    /// 11, or 13. The smallest versions are `Version::RectMicro(7, 43)` of size
    /// 7×43 when the height is minimum and `Version::RectMicro(11, 27)` of size
    /// 11×27 when the width is minimum, and the largest is
    /// `Version::RectMicro(17, 139)` of size 17×139.
    RectMicro(i16, i16),
}

impl Version {
    /// Gets the number of horizontally-arranged "modules" on each size of the
    /// QR code, i.e. the width of the code.
    ///
    /// Except for rMQR code, the width is the same as the height.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::Version;
    /// #
    /// assert_eq!(Version::Normal(40).width(), 177);
    /// assert_eq!(Version::Micro(4).width(), 17);
    /// assert_eq!(Version::RectMicro(17, 139).width(), 139);
    /// ```
    #[must_use]
    pub const fn width(self) -> i16 {
        match self {
            Self::Normal(v) => v * 4 + 17,
            Self::Micro(v) => v * 2 + 9,
            Self::RectMicro(_, w) => w,
        }
    }

    /// Gets the number of vertically-arranged "modules" on each size of the QR
    /// code, i.e. the height of the code.
    ///
    /// Except for rMQR code, the height is the same as the width.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::Version;
    /// #
    /// assert_eq!(Version::Normal(40).height(), 177);
    /// assert_eq!(Version::Micro(4).height(), 17);
    /// assert_eq!(Version::RectMicro(17, 139).height(), 17);
    /// ```
    #[must_use]
    pub const fn height(self) -> i16 {
        if let Self::RectMicro(h, _) = self {
            h
        } else {
            self.width()
        }
    }

    /// Obtains an object from a hard-coded table.
    ///
    /// The table must be a 76×4 array. The outer array represents the content
    /// for each version. The first 40 entry corresponds to QR code versions 1
    /// to 40, the next 4 corresponds to Micro QR code version 1 to 4, and the
    /// last 32 corresponds to rMQR code. The inner array represents the content
    /// in each error correction level, in the order [L, M, Q, H].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the entry compares equal to the default value of `T`.
    pub fn fetch<T>(self, ec_level: EcLevel, table: &[[T; 4]]) -> QrResult<T>
    where
        T: Copy + Default + PartialEq,
    {
        match self {
            Self::Normal(v @ 1..=40) => {
                return Ok(table[(v - 1).as_usize()][ec_level as usize]);
            }
            Self::Micro(v @ 1..=4) => {
                let obj = table[(v + 39).as_usize()][ec_level as usize];
                if obj != T::default() {
                    return Ok(obj);
                }
            }
            Self::RectMicro(..) => {
                let index = self.rect_micro_index()?;
                let obj = table[index + 44][ec_level as usize];
                if obj != T::default() {
                    return Ok(obj);
                }
            }
            _ => {}
        }
        Err(QrError::InvalidVersion)
    }

    /// Returns the number of bits needed to encode the mode indicator.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::Version;
    /// #
    /// assert_eq!(Version::Normal(40).mode_bits_count(), 4);
    /// assert_eq!(Version::Micro(4).mode_bits_count(), 3);
    /// assert_eq!(Version::RectMicro(17, 139).mode_bits_count(), 3);
    /// ```
    #[must_use]
    pub fn mode_bits_count(self) -> usize {
        match self {
            Self::Normal(_) => 4,
            Self::Micro(a) => (a - 1).as_usize(),
            Self::RectMicro(..) => 3,
        }
    }

    /// Checks whether is version refers to a normal QR code.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::Version;
    /// #
    /// assert_eq!(Version::Normal(1).is_normal(), true);
    /// assert_eq!(Version::Normal(40).is_normal(), true);
    /// // Invalid normal QR code version.
    /// assert_eq!(Version::Normal(0).is_normal(), false);
    ///
    /// assert_eq!(Version::Micro(1).is_normal(), false);
    /// assert_eq!(Version::RectMicro(7, 43).is_normal(), false);
    /// ```
    #[must_use]
    pub const fn is_normal(self) -> bool {
        matches!(self, Self::Normal(v) if v >= 1 && v <= 40)
    }

    /// Checks whether is version refers to a Micro QR code.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::Version;
    /// #
    /// assert_eq!(Version::Micro(1).is_micro(), true);
    /// assert_eq!(Version::Micro(4).is_micro(), true);
    /// // Invalid Micro QR code version.
    /// assert_eq!(Version::Micro(0).is_micro(), false);
    ///
    /// assert_eq!(Version::Normal(1).is_micro(), false);
    /// assert_eq!(Version::RectMicro(7, 43).is_micro(), false);
    /// ```
    #[must_use]
    pub const fn is_micro(self) -> bool {
        matches!(self, Self::Micro(v) if v >= 1 && v <= 4)
    }

    /// Checks whether is version refers to a rMQR code.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::Version;
    /// #
    /// assert_eq!(Version::RectMicro(7, 43).is_rect_micro(), true);
    /// assert_eq!(Version::RectMicro(17, 139).is_rect_micro(), true);
    /// // Invalid rMQR code version.
    /// assert_eq!(Version::RectMicro(0, 0).is_rect_micro(), false);
    ///
    /// assert_eq!(Version::Normal(1).is_rect_micro(), false);
    /// assert_eq!(Version::Micro(1).is_rect_micro(), false);
    /// ```
    #[must_use]
    pub const fn is_rect_micro(self) -> bool {
        self.rect_micro_index().is_ok()
    }

    /// Gets the index of the version of the rMQR code.
    pub(crate) const fn rect_micro_index(self) -> QrResult<usize> {
        match self {
            Self::RectMicro(7, 43) => Ok(0),
            Self::RectMicro(7, 59) => Ok(1),
            Self::RectMicro(7, 77) => Ok(2),
            Self::RectMicro(7, 99) => Ok(3),
            Self::RectMicro(7, 139) => Ok(4),
            Self::RectMicro(9, 43) => Ok(5),
            Self::RectMicro(9, 59) => Ok(6),
            Self::RectMicro(9, 77) => Ok(7),
            Self::RectMicro(9, 99) => Ok(8),
            Self::RectMicro(9, 139) => Ok(9),
            Self::RectMicro(11, 27) => Ok(10),
            Self::RectMicro(11, 43) => Ok(11),
            Self::RectMicro(11, 59) => Ok(12),
            Self::RectMicro(11, 77) => Ok(13),
            Self::RectMicro(11, 99) => Ok(14),
            Self::RectMicro(11, 139) => Ok(15),
            Self::RectMicro(13, 27) => Ok(16),
            Self::RectMicro(13, 43) => Ok(17),
            Self::RectMicro(13, 59) => Ok(18),
            Self::RectMicro(13, 77) => Ok(19),
            Self::RectMicro(13, 99) => Ok(20),
            Self::RectMicro(13, 139) => Ok(21),
            Self::RectMicro(15, 43) => Ok(22),
            Self::RectMicro(15, 59) => Ok(23),
            Self::RectMicro(15, 77) => Ok(24),
            Self::RectMicro(15, 99) => Ok(25),
            Self::RectMicro(15, 139) => Ok(26),
            Self::RectMicro(17, 43) => Ok(27),
            Self::RectMicro(17, 59) => Ok(28),
            Self::RectMicro(17, 77) => Ok(29),
            Self::RectMicro(17, 99) => Ok(30),
            Self::RectMicro(17, 139) => Ok(31),
            _ => Err(QrError::InvalidVersion),
        }
    }

    /// Gets the index in ascending order of width.
    pub(crate) const fn rect_micro_width_index(self) -> QrResult<usize> {
        match self {
            Self::RectMicro(_, 27) => Ok(0),
            Self::RectMicro(_, 43) => Ok(1),
            Self::RectMicro(_, 59) => Ok(2),
            Self::RectMicro(_, 77) => Ok(3),
            Self::RectMicro(_, 99) => Ok(4),
            Self::RectMicro(_, 139) => Ok(5),
            _ => Err(QrError::InvalidVersion),
        }
    }

    /// All widths of rMQR code.
    pub(crate) const RMQR_ALL_WIDTH: [i16; 6] = [27, 43, 59, 77, 99, 139];

    /// All heights of rMQR code.
    pub(crate) const RMQR_ALL_HEIGHT: [i16; 6] = [7, 9, 11, 13, 15, 17];
}

#[cfg(test)]
mod version_tests {
    use super::*;

    #[test]
    fn test_width() {
        assert_eq!(Version::Normal(1).width(), 21);
        assert_eq!(Version::Normal(40).width(), 177);
        assert_eq!(Version::Micro(1).width(), 11);
        assert_eq!(Version::Micro(4).width(), 17);
        assert_eq!(Version::RectMicro(7, 43).width(), 43);
        assert_eq!(Version::RectMicro(11, 27).width(), 27);
        assert_eq!(Version::RectMicro(17, 139).width(), 139);
    }

    #[test]
    fn test_height() {
        assert_eq!(Version::Normal(1).height(), 21);
        assert_eq!(Version::Normal(40).height(), 177);
        assert_eq!(Version::Micro(1).height(), 11);
        assert_eq!(Version::Micro(4).height(), 17);
        assert_eq!(Version::RectMicro(7, 43).height(), 7);
        assert_eq!(Version::RectMicro(11, 27).height(), 11);
        assert_eq!(Version::RectMicro(17, 139).height(), 17);
    }

    #[test]
    fn test_mode_bits_count() {
        assert_eq!(Version::Normal(1).mode_bits_count(), 4);
        for version in 1..=4 {
            assert_eq!(
                Version::Micro(version).mode_bits_count(),
                (version - 1).as_usize()
            );
        }
        assert_eq!(Version::RectMicro(7, 43).mode_bits_count(), 3);
    }

    #[test]
    fn test_is_normal() {
        for version in 1..=40 {
            assert!(Version::Normal(version).is_normal());
        }
        assert!(!Version::Normal(0).is_normal());
        assert!(!Version::Normal(41).is_normal());

        assert!(!Version::Micro(1).is_normal());
        assert!(!Version::RectMicro(7, 43).is_normal());
    }

    #[test]
    fn test_is_micro() {
        for version in 1..=4 {
            assert!(Version::Micro(version).is_micro());
        }
        assert!(!Version::Micro(0).is_micro());
        assert!(!Version::Micro(5).is_micro());

        assert!(!Version::Normal(1).is_micro());
        assert!(!Version::RectMicro(7, 43).is_micro());
    }

    #[test]
    fn test_is_rect_micro() {
        for width in Version::RMQR_ALL_WIDTH {
            for height in Version::RMQR_ALL_HEIGHT {
                if width == 27 && (height != 11 && height != 13) {
                    continue;
                }
                assert!(Version::RectMicro(height, width).is_rect_micro());
            }
        }
        assert!(!Version::RectMicro(0, 0).is_rect_micro());

        assert!(!Version::Normal(1).is_rect_micro());
        assert!(!Version::Micro(1).is_rect_micro());
    }
}

// Mode indicator

/// The mode indicator, which specifies the character set of the encoded data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    /// The data contains only characters 0 to 9.
    Numeric,

    /// The data contains only uppercase letters (A–Z), numbers (0–9) and a few
    /// punctuations marks (space, `$`, `%`, `*`, `+`, `-`, `.`, `/`, `:`).
    Alphanumeric,

    /// The data contains arbitrary binary data.
    Byte,

    /// The data contains Shift-JIS-encoded double-byte text.
    Kanji,
}

impl Mode {
    /// Computes the number of bits needed to encode the data length.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the mode is not supported in the given version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::{Version, types::Mode};
    /// #
    /// assert_eq!(Mode::Numeric.length_bits_count(Version::Normal(1)), 10);
    /// ```
    #[must_use]
    pub fn length_bits_count(self, version: Version) -> usize {
        match version {
            Version::Micro(a) => {
                let a = a.as_usize();
                match self {
                    Self::Numeric => 2 + a,
                    Self::Alphanumeric | Self::Byte => 1 + a,
                    Self::Kanji => a,
                }
            }
            Version::Normal(1..=9) => match self {
                Self::Numeric => 10,
                Self::Alphanumeric => 9,
                Self::Byte | Self::Kanji => 8,
            },
            Version::Normal(10..=26) => match self {
                Self::Numeric => 12,
                Self::Alphanumeric => 11,
                Self::Byte => 16,
                Self::Kanji => 10,
            },
            Version::Normal(_) => match self {
                Self::Numeric => 14,
                Self::Alphanumeric => 13,
                Self::Byte => 16,
                Self::Kanji => 12,
            },
            Version::RectMicro(..) => {
                let index = version.rect_micro_index().unwrap_or(31);
                match self {
                    Self::Numeric => RMQR_LENGTH_BITS_COUNT[index][0],
                    Self::Alphanumeric => RMQR_LENGTH_BITS_COUNT[index][1],
                    Self::Byte => RMQR_LENGTH_BITS_COUNT[index][2],
                    Self::Kanji => RMQR_LENGTH_BITS_COUNT[index][3],
                }
            }
        }
    }

    /// Computes the number of bits needed to some data of a given raw length.
    ///
    /// <div class="warning">
    ///
    /// Note that in Kanji mode, the `raw_data_len` is the number of Kanjis,
    /// i.e. half the total size of bytes.
    ///
    /// </div>
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::types::Mode;
    /// #
    /// assert_eq!(Mode::Numeric.data_bits_count(7), 24);
    /// ```
    #[must_use]
    pub const fn data_bits_count(self, raw_data_len: usize) -> usize {
        match self {
            Self::Numeric => (raw_data_len * 10).div_ceil(3),
            Self::Alphanumeric => (raw_data_len * 11).div_ceil(2),
            Self::Byte => raw_data_len * 8,
            Self::Kanji => raw_data_len * 13,
        }
    }

    /// Finds the lowest common mode which both modes are compatible with.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::types::Mode;
    /// #
    /// let a = Mode::Numeric;
    /// let b = Mode::Kanji;
    /// let c = a.max(b);
    /// assert!(a <= c);
    /// assert!(b <= c);
    /// ```
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        match self.partial_cmp(&other) {
            Some(Ordering::Greater) => self,
            Some(_) => other,
            None => Self::Byte,
        }
    }
}

impl PartialOrd for Mode {
    /// Defines a partial ordering between modes. If `self <= other`, then
    /// `other` contains a superset of all characters supported by `self`.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (*self, *other) {
            (a, b) if a == b => Some(Ordering::Equal),
            (Self::Numeric, Self::Alphanumeric) | (_, Self::Byte) => Some(Ordering::Less),
            (Self::Alphanumeric, Self::Numeric) | (Self::Byte, _) => Some(Ordering::Greater),
            _ => None,
        }
    }
}

#[cfg(test)]
mod mode_tests {
    use super::*;

    #[test]
    fn test_mode_order() {
        assert!(Mode::Numeric < Mode::Alphanumeric);
        assert!(Mode::Byte > Mode::Kanji);
        assert!(!(Mode::Numeric < Mode::Kanji));
        assert!(!(Mode::Numeric >= Mode::Kanji));
    }

    #[test]
    fn test_max() {
        assert_eq!(Mode::Byte.max(Mode::Kanji), Mode::Byte);
        assert_eq!(Mode::Numeric.max(Mode::Alphanumeric), Mode::Alphanumeric);
        assert_eq!(
            Mode::Alphanumeric.max(Mode::Alphanumeric),
            Mode::Alphanumeric
        );
        assert_eq!(Mode::Numeric.max(Mode::Kanji), Mode::Byte);
        assert_eq!(Mode::Kanji.max(Mode::Numeric), Mode::Byte);
        assert_eq!(Mode::Alphanumeric.max(Mode::Numeric), Mode::Alphanumeric);
        assert_eq!(Mode::Kanji.max(Mode::Kanji), Mode::Kanji);
    }
}

/// The number of bits needed to encode the length of the data.
///
/// [Numeric, Alphanumeric, Byte, Kanji]
static RMQR_LENGTH_BITS_COUNT: [[usize; 4]; 32] = [
    // R7x43
    [4, 3, 3, 2],
    // R7x59
    [5, 5, 4, 3],
    // R7x77
    [6, 5, 5, 4],
    // R7x99
    [7, 6, 5, 5],
    // R7x139
    [7, 6, 6, 5],
    // R9x43
    [5, 5, 4, 3],
    // R9x59
    [6, 5, 5, 4],
    // R9x77
    [7, 6, 5, 5],
    // R9x99
    [7, 6, 6, 5],
    // R9x139
    [8, 7, 6, 6],
    // R11x27
    [4, 4, 3, 2],
    // R11x43
    [6, 5, 5, 4],
    // R11x59
    [7, 6, 5, 5],
    // R11x77
    [7, 6, 6, 5],
    // R11x99
    [8, 7, 6, 6],
    // R11x139
    [8, 7, 7, 6],
    // R13x27
    [5, 5, 4, 3],
    // R13x43
    [6, 6, 5, 5],
    // R13x59
    [7, 6, 6, 5],
    // R13x77
    [7, 7, 6, 6],
    // R13x99
    [8, 7, 7, 6],
    // R13x139
    [8, 8, 7, 7],
    // R15x43
    [7, 6, 6, 5],
    // R15x59
    [7, 7, 6, 5],
    // R15x77
    [8, 7, 7, 6],
    // R15x99
    [8, 7, 7, 6],
    // R15x139
    [9, 8, 7, 7],
    // R17x43
    [7, 6, 6, 5],
    // R17x59
    [8, 7, 6, 6],
    // R17x77
    [8, 7, 7, 6],
    // R17x99
    [8, 8, 7, 6],
    // R17x139
    [9, 8, 8, 7],
];
