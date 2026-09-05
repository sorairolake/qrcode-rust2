// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`Version`].

mod micro;
mod normal;
mod rect_micro;

pub use self::{micro::MicroVersion, normal::NormalVersion, rect_micro::RectMicroVersion};
use super::EcLevel;
use crate::error::{Error, Result};

/// In QR code terminology, `Version` means the size of the generated image.
/// Larger version means the size of code is larger, and therefore can carry
/// more information.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Version {
    /// A QR code model 2 version.
    Normal(NormalVersion),

    /// A Micro QR code version.
    Micro(MicroVersion),

    /// A rMQR code version.
    RectMicro(RectMicroVersion),
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
    /// use qrcode2::{MicroVersion, NormalVersion, RectMicroVersion, Version};
    ///
    /// assert_eq!(Version::Normal(NormalVersion::V40).width(), 177);
    /// assert_eq!(Version::Micro(MicroVersion::M4).width(), 17);
    /// assert_eq!(Version::RectMicro(RectMicroVersion::R17x139).width(), 139);
    /// ```
    #[must_use]
    pub fn width(self) -> u8 {
        match self {
            Self::Normal(v) => u8::from(v) * 4 + 17,
            Self::Micro(v) => u8::from(v) * 2 + 9,
            Self::RectMicro(v) => {
                let (_, w) = <(u8, u8)>::from(v);
                w
            }
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
    /// use qrcode2::{MicroVersion, NormalVersion, RectMicroVersion, Version};
    ///
    /// assert_eq!(Version::Normal(NormalVersion::V40).height(), 177);
    /// assert_eq!(Version::Micro(MicroVersion::M4).height(), 17);
    /// assert_eq!(Version::RectMicro(RectMicroVersion::R17x139).height(), 17);
    /// ```
    #[must_use]
    pub fn height(self) -> u8 {
        if let Self::RectMicro(v) = self {
            let (h, _) = <(u8, u8)>::from(v);
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
            Self::Normal(v) => Ok(table[usize::from(u8::from(v) - 1)][ec_level as usize]),
            Self::Micro(v) => {
                let obj = table[usize::from(u8::from(v) + 39)][ec_level as usize];
                if obj == T::default() {
                    Err(Error::InvalidVersion)
                } else {
                    Ok(obj)
                }
            }
            Self::RectMicro(v) => {
                let index = v.index();
                let obj = table[index + 44][ec_level as usize];
                if obj == T::default() {
                    Err(Error::InvalidVersion)
                } else {
                    Ok(obj)
                }
            }
        }
    }

    /// Returns the number of bits needed to encode the mode indicator.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{MicroVersion, NormalVersion, RectMicroVersion, Version};
    ///
    /// assert_eq!(Version::Normal(NormalVersion::V40).mode_bits_count(), 4);
    /// assert_eq!(Version::Micro(MicroVersion::M4).mode_bits_count(), 3);
    /// assert_eq!(
    ///     Version::RectMicro(RectMicroVersion::R17x139).mode_bits_count(),
    ///     3
    /// );
    /// ```
    #[must_use]
    pub fn mode_bits_count(self) -> usize {
        match self {
            Self::Normal(_) => 4,
            Self::Micro(a) => (u8::from(a) - 1).into(),
            Self::RectMicro(_) => 3,
        }
    }

    /// Checks whether is version refers to a QR code model 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{MicroVersion, NormalVersion, RectMicroVersion, Version};
    ///
    /// assert_eq!(Version::Normal(NormalVersion::V1).is_normal(), true);
    /// assert_eq!(Version::Normal(NormalVersion::V40).is_normal(), true);
    ///
    /// assert_eq!(Version::Micro(MicroVersion::M1).is_normal(), false);
    /// assert_eq!(
    ///     Version::RectMicro(RectMicroVersion::R7x43).is_normal(),
    ///     false
    /// );
    /// ```
    #[must_use]
    pub const fn is_normal(self) -> bool {
        matches!(self, Self::Normal(_))
    }

    /// Checks whether is version refers to a Micro QR code.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{MicroVersion, NormalVersion, RectMicroVersion, Version};
    ///
    /// assert_eq!(Version::Micro(MicroVersion::M1).is_micro(), true);
    /// assert_eq!(Version::Micro(MicroVersion::M4).is_micro(), true);
    ///
    /// assert_eq!(Version::Normal(NormalVersion::V1).is_micro(), false);
    /// assert_eq!(
    ///     Version::RectMicro(RectMicroVersion::R7x43).is_micro(),
    ///     false
    /// );
    /// ```
    #[must_use]
    pub const fn is_micro(self) -> bool {
        matches!(self, Self::Micro(_))
    }

    /// Checks whether is version refers to a rMQR code.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{MicroVersion, NormalVersion, RectMicroVersion, Version};
    ///
    /// assert_eq!(
    ///     Version::RectMicro(RectMicroVersion::R7x43).is_rect_micro(),
    ///     true
    /// );
    /// assert_eq!(
    ///     Version::RectMicro(RectMicroVersion::R17x139).is_rect_micro(),
    ///     true
    /// );
    ///
    /// assert_eq!(Version::Normal(NormalVersion::V1).is_rect_micro(), false);
    /// assert_eq!(Version::Micro(MicroVersion::M1).is_rect_micro(), false);
    /// ```
    #[must_use]
    pub const fn is_rect_micro(self) -> bool {
        matches!(self, Self::RectMicro(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width() {
        assert_eq!(Version::Normal(NormalVersion::V1).width(), 21);
        assert_eq!(Version::Normal(NormalVersion::V40).width(), 177);
        assert_eq!(Version::Micro(MicroVersion::M1).width(), 11);
        assert_eq!(Version::Micro(MicroVersion::M4).width(), 17);
        assert_eq!(Version::RectMicro(RectMicroVersion::R7x43).width(), 43);
        assert_eq!(Version::RectMicro(RectMicroVersion::R11x27).width(), 27);
        assert_eq!(Version::RectMicro(RectMicroVersion::R17x139).width(), 139);
    }

    #[test]
    fn height() {
        assert_eq!(Version::Normal(NormalVersion::V1).height(), 21);
        assert_eq!(Version::Normal(NormalVersion::V40).height(), 177);
        assert_eq!(Version::Micro(MicroVersion::M1).height(), 11);
        assert_eq!(Version::Micro(MicroVersion::M4).height(), 17);
        assert_eq!(Version::RectMicro(RectMicroVersion::R7x43).height(), 7);
        assert_eq!(Version::RectMicro(RectMicroVersion::R11x27).height(), 11);
        assert_eq!(Version::RectMicro(RectMicroVersion::R17x139).height(), 17);
    }

    #[test]
    fn mode_bits_count() {
        assert_eq!(Version::Normal(NormalVersion::V1).mode_bits_count(), 4);
        for version in MicroVersion::ALL {
            assert_eq!(
                Version::Micro(version).mode_bits_count(),
                (u8::from(version) - 1).into()
            );
        }
        assert_eq!(
            Version::RectMicro(RectMicroVersion::R7x43).mode_bits_count(),
            3
        );
    }

    #[test]
    fn is_normal() {
        let all_versions = [
            NormalVersion::V1,
            NormalVersion::V2,
            NormalVersion::V3,
            NormalVersion::V4,
            NormalVersion::V5,
            NormalVersion::V6,
            NormalVersion::V7,
            NormalVersion::V8,
            NormalVersion::V9,
            NormalVersion::V10,
            NormalVersion::V11,
            NormalVersion::V12,
            NormalVersion::V13,
            NormalVersion::V14,
            NormalVersion::V15,
            NormalVersion::V16,
            NormalVersion::V17,
            NormalVersion::V18,
            NormalVersion::V19,
            NormalVersion::V20,
            NormalVersion::V21,
            NormalVersion::V22,
            NormalVersion::V23,
            NormalVersion::V24,
            NormalVersion::V25,
            NormalVersion::V26,
            NormalVersion::V27,
            NormalVersion::V28,
            NormalVersion::V29,
            NormalVersion::V30,
            NormalVersion::V31,
            NormalVersion::V32,
            NormalVersion::V33,
            NormalVersion::V34,
            NormalVersion::V35,
            NormalVersion::V36,
            NormalVersion::V37,
            NormalVersion::V38,
            NormalVersion::V39,
            NormalVersion::V40,
        ];
        for version in all_versions {
            assert!(Version::Normal(version).is_normal());
        }

        assert!(!Version::Micro(MicroVersion::M1).is_normal());
        assert!(!Version::RectMicro(RectMicroVersion::R7x43).is_normal());
    }

    #[test]
    fn is_micro() {
        for version in MicroVersion::ALL {
            assert!(Version::Micro(version).is_micro());
        }

        assert!(!Version::Normal(NormalVersion::V1).is_micro());
        assert!(!Version::RectMicro(RectMicroVersion::R7x43).is_micro());
    }

    #[test]
    fn is_rect_micro() {
        for version in RectMicroVersion::ALL {
            assert!(Version::RectMicro(version).is_rect_micro());
        }

        assert!(!Version::Normal(NormalVersion::V1).is_rect_micro());
        assert!(!Version::Micro(MicroVersion::M1).is_rect_micro());
    }
}
