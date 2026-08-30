// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`Version`].

use super::EcLevel;
use crate::{
    cast::As,
    error::{Error, Result},
};

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
    /// use qrcode2::Version;
    ///
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
    /// use qrcode2::Version;
    ///
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
    pub fn fetch<T>(self, ec_level: EcLevel, table: &[[T; 4]]) -> Result<T>
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
        Err(Error::InvalidVersion)
    }

    /// Returns the number of bits needed to encode the mode indicator.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::Version;
    ///
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
    /// use qrcode2::Version;
    ///
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
    /// use qrcode2::Version;
    ///
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
    /// use qrcode2::Version;
    ///
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
    pub(crate) const fn rect_micro_index(self) -> Result<usize> {
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
            _ => Err(Error::InvalidVersion),
        }
    }

    /// Gets the index in ascending order of width.
    pub(crate) const fn rect_micro_width_index(self) -> Result<usize> {
        match self {
            Self::RectMicro(_, 27) => Ok(0),
            Self::RectMicro(_, 43) => Ok(1),
            Self::RectMicro(_, 59) => Ok(2),
            Self::RectMicro(_, 77) => Ok(3),
            Self::RectMicro(_, 99) => Ok(4),
            Self::RectMicro(_, 139) => Ok(5),
            _ => Err(Error::InvalidVersion),
        }
    }

    /// All widths of rMQR code.
    pub(crate) const RMQR_ALL_WIDTH: [i16; 6] = [27, 43, 59, 77, 99, 139];

    /// All heights of rMQR code.
    pub(crate) const RMQR_ALL_HEIGHT: [i16; 6] = [7, 9, 11, 13, 15, 17];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width() {
        assert_eq!(Version::Normal(1).width(), 21);
        assert_eq!(Version::Normal(40).width(), 177);
        assert_eq!(Version::Micro(1).width(), 11);
        assert_eq!(Version::Micro(4).width(), 17);
        assert_eq!(Version::RectMicro(7, 43).width(), 43);
        assert_eq!(Version::RectMicro(11, 27).width(), 27);
        assert_eq!(Version::RectMicro(17, 139).width(), 139);
    }

    #[test]
    fn height() {
        assert_eq!(Version::Normal(1).height(), 21);
        assert_eq!(Version::Normal(40).height(), 177);
        assert_eq!(Version::Micro(1).height(), 11);
        assert_eq!(Version::Micro(4).height(), 17);
        assert_eq!(Version::RectMicro(7, 43).height(), 7);
        assert_eq!(Version::RectMicro(11, 27).height(), 11);
        assert_eq!(Version::RectMicro(17, 139).height(), 17);
    }

    #[test]
    fn mode_bits_count() {
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
    fn is_normal() {
        for version in 1..=40 {
            assert!(Version::Normal(version).is_normal());
        }
        assert!(!Version::Normal(0).is_normal());
        assert!(!Version::Normal(41).is_normal());

        assert!(!Version::Micro(1).is_normal());
        assert!(!Version::RectMicro(7, 43).is_normal());
    }

    #[test]
    fn is_micro() {
        for version in 1..=4 {
            assert!(Version::Micro(version).is_micro());
        }
        assert!(!Version::Micro(0).is_micro());
        assert!(!Version::Micro(5).is_micro());

        assert!(!Version::Normal(1).is_micro());
        assert!(!Version::RectMicro(7, 43).is_micro());
    }

    #[test]
    fn is_rect_micro() {
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
