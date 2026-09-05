// SPDX-FileCopyrightText: 2026 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`MicroVersion`].

use crate::error::Error;

/// `MicroVersion` is a type that represents a Micro QR code version.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MicroVersion {
    /// An 11×11 Micro QR code symbol.
    M1 = 1,

    /// A 13×13 Micro QR code symbol.
    M2,

    /// A 15×15 Micro QR code symbol.
    M3,

    /// A 17×17 Micro QR code symbol.
    M4,
}

impl MicroVersion {
    /// The smallest Micro QR code version.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::MicroVersion;
    ///
    /// assert_eq!(MicroVersion::MIN, MicroVersion::M1);
    /// ```
    pub const MIN: Self = Self::M1;

    /// The largest Micro QR code version.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::MicroVersion;
    ///
    /// assert_eq!(MicroVersion::MAX, MicroVersion::M4);
    /// ```
    pub const MAX: Self = Self::M4;

    /// All versions of Micro QR code.
    pub(crate) const ALL: [Self; 4] = [Self::M1, Self::M2, Self::M3, Self::M4];

    /// Gets the number of modules on each side.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::MicroVersion;
    ///
    /// assert_eq!(MicroVersion::M1.size(), 11);
    /// assert_eq!(MicroVersion::M4.size(), 17);
    /// ```
    #[must_use]
    pub fn size(self) -> u8 {
        u8::from(self) * 2 + 9
    }

    /// Returns the number of bits needed to encode the mode indicator.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::MicroVersion;
    ///
    /// assert_eq!(MicroVersion::M1.mode_bits_count(), 0);
    /// assert_eq!(MicroVersion::M4.mode_bits_count(), 3);
    /// ```
    #[must_use]
    pub fn mode_bits_count(self) -> usize {
        (u8::from(self) - 1).into()
    }
}

impl From<MicroVersion> for u8 {
    /// Converts a `MicroVersion` into a [`u8`] value.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::MicroVersion;
    ///
    /// assert_eq!(u8::from(MicroVersion::M1), 1);
    /// assert_eq!(u8::from(MicroVersion::M4), 4);
    /// ```
    fn from(version: MicroVersion) -> Self {
        version as Self
    }
}

impl TryFrom<u8> for MicroVersion {
    type Error = Error;

    /// Converts a [`u8`] value to a `MicroVersion`.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if `version` is not a valid Micro QR code version.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::MicroVersion;
    ///
    /// assert_eq!(MicroVersion::try_from(1), Ok(MicroVersion::M1));
    /// assert_eq!(MicroVersion::try_from(4), Ok(MicroVersion::M4));
    ///
    /// assert!(MicroVersion::try_from(0).is_err());
    /// ```
    fn try_from(version: u8) -> Result<Self, Self::Error> {
        match version {
            1 => Ok(Self::M1),
            2 => Ok(Self::M2),
            3 => Ok(Self::M3),
            4 => Ok(Self::M4),
            _ => Err(Error::InvalidVersion),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_micro_version_to_u8() {
        assert_eq!(u8::from(MicroVersion::M1), 1);
        assert_eq!(u8::from(MicroVersion::M2), 2);
        assert_eq!(u8::from(MicroVersion::M3), 3);
        assert_eq!(u8::from(MicroVersion::M4), 4);
    }

    #[test]
    fn try_from_u8_to_micro_version() {
        assert_eq!(MicroVersion::try_from(1).unwrap(), MicroVersion::M1);
        assert_eq!(MicroVersion::try_from(2).unwrap(), MicroVersion::M2);
        assert_eq!(MicroVersion::try_from(3).unwrap(), MicroVersion::M3);
        assert_eq!(MicroVersion::try_from(4).unwrap(), MicroVersion::M4);
    }

    #[test]
    fn try_from_u8_to_micro_version_with_invalid_version() {
        assert_eq!(
            MicroVersion::try_from(0).unwrap_err(),
            Error::InvalidVersion
        );
        assert_eq!(
            MicroVersion::try_from(5).unwrap_err(),
            Error::InvalidVersion
        );
        assert_eq!(
            MicroVersion::try_from(u8::MAX).unwrap_err(),
            Error::InvalidVersion
        );
    }
}
