// SPDX-FileCopyrightText: 2026 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`RectMicroVersion`].

use crate::error::Error;

/// `RectMicroVersion` is a type that represents a rMQR code version.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RectMicroVersion {
    /// A 7×43 rMQR code symbol.
    R7x43,

    /// A 7×59 rMQR code symbol.
    R7x59,

    /// A 7×77 rMQR code symbol.
    R7x77,

    /// A 7×99 rMQR code symbol.
    R7x99,

    /// A 7×139 rMQR code symbol.
    R7x139,

    /// A 9×43 rMQR code symbol.
    R9x43,

    /// A 9×59 rMQR code symbol.
    R9x59,

    /// A 9×77 rMQR code symbol.
    R9x77,

    /// A 9×99 rMQR code symbol.
    R9x99,

    /// A 9×139 rMQR code symbol.
    R9x139,

    /// An 11×27 rMQR code symbol.
    R11x27,

    /// An 11×43 rMQR code symbol.
    R11x43,

    /// An 11×59 rMQR code symbol.
    R11x59,

    /// An 11×77 rMQR code symbol.
    R11x77,

    /// An 11×99 rMQR code symbol.
    R11x99,

    /// An 11×139 rMQR code symbol.
    R11x139,

    /// A 13×27 rMQR code symbol.
    R13x27,

    /// A 13×43 rMQR code symbol.
    R13x43,

    /// A 13×59 rMQR code symbol.
    R13x59,

    /// A 13×77 rMQR code symbol.
    R13x77,

    /// A 13×99 rMQR code symbol.
    R13x99,

    /// A 13×139 rMQR code symbol.
    R13x139,

    /// A 15×43 rMQR code symbol.
    R15x43,

    /// A 15×59 rMQR code symbol.
    R15x59,

    /// A 15×77 rMQR code symbol.
    R15x77,

    /// A 15×99 rMQR code symbol.
    R15x99,

    /// A 15×139 rMQR code symbol.
    R15x139,

    /// A 17×43 rMQR code symbol.
    R17x43,

    /// A 17×59 rMQR code symbol.
    R17x59,

    /// A 17×77 rMQR code symbol.
    R17x77,

    /// A 17×99 rMQR code symbol.
    R17x99,

    /// A 17×139 rMQR code symbol.
    R17x139,
}

impl RectMicroVersion {
    /// The first rMQR code version in the version indicator order.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::RectMicroVersion;
    ///
    /// assert_eq!(RectMicroVersion::MIN, RectMicroVersion::R7x43);
    /// ```
    pub const MIN: Self = Self::R7x43;

    /// The last rMQR code version in the version indicator order.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::RectMicroVersion;
    ///
    /// assert_eq!(RectMicroVersion::MAX, RectMicroVersion::R17x139);
    /// ```
    pub const MAX: Self = Self::R17x139;

    /// All versions of rMQR code.
    pub(crate) const ALL: [Self; 32] = [
        Self::R7x43,
        Self::R7x59,
        Self::R7x77,
        Self::R7x99,
        Self::R7x139,
        Self::R9x43,
        Self::R9x59,
        Self::R9x77,
        Self::R9x99,
        Self::R9x139,
        Self::R11x27,
        Self::R11x43,
        Self::R11x59,
        Self::R11x77,
        Self::R11x99,
        Self::R11x139,
        Self::R13x27,
        Self::R13x43,
        Self::R13x59,
        Self::R13x77,
        Self::R13x99,
        Self::R13x139,
        Self::R15x43,
        Self::R15x59,
        Self::R15x77,
        Self::R15x99,
        Self::R15x139,
        Self::R17x43,
        Self::R17x59,
        Self::R17x77,
        Self::R17x99,
        Self::R17x139,
    ];

    /// Gets the number of horizontally-arranged "modules" on each size of the
    /// rMQR code, i.e. the width of the code.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::RectMicroVersion;
    ///
    /// assert_eq!(RectMicroVersion::R7x43.width(), 43);
    /// assert_eq!(RectMicroVersion::R17x139.width(), 139);
    /// ```
    #[must_use]
    pub fn width(self) -> u8 {
        let (_, w) = <(u8, u8)>::from(self);
        w
    }

    /// Gets the number of vertically-arranged "modules" on each size of the
    /// rMQR code, i.e. the height of the code.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::RectMicroVersion;
    ///
    /// assert_eq!(RectMicroVersion::R7x43.height(), 7);
    /// assert_eq!(RectMicroVersion::R17x139.height(), 17);
    /// ```
    #[must_use]
    pub fn height(self) -> u8 {
        let (h, _) = <(u8, u8)>::from(self);
        h
    }

    /// Returns the number of bits needed to encode the mode indicator.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::RectMicroVersion;
    ///
    /// assert_eq!(RectMicroVersion::R7x43.mode_bits_count(), 3);
    /// assert_eq!(RectMicroVersion::R17x139.mode_bits_count(), 3);
    /// ```
    #[must_use]
    pub const fn mode_bits_count(self) -> u8 {
        3
    }

    /// Gets the index of the version of the rMQR code.
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::R7x43 => 0,
            Self::R7x59 => 1,
            Self::R7x77 => 2,
            Self::R7x99 => 3,
            Self::R7x139 => 4,
            Self::R9x43 => 5,
            Self::R9x59 => 6,
            Self::R9x77 => 7,
            Self::R9x99 => 8,
            Self::R9x139 => 9,
            Self::R11x27 => 10,
            Self::R11x43 => 11,
            Self::R11x59 => 12,
            Self::R11x77 => 13,
            Self::R11x99 => 14,
            Self::R11x139 => 15,
            Self::R13x27 => 16,
            Self::R13x43 => 17,
            Self::R13x59 => 18,
            Self::R13x77 => 19,
            Self::R13x99 => 20,
            Self::R13x139 => 21,
            Self::R15x43 => 22,
            Self::R15x59 => 23,
            Self::R15x77 => 24,
            Self::R15x99 => 25,
            Self::R15x139 => 26,
            Self::R17x43 => 27,
            Self::R17x59 => 28,
            Self::R17x77 => 29,
            Self::R17x99 => 30,
            Self::R17x139 => 31,
        }
    }

    /// Gets the index in ascending order of width.
    pub(crate) const fn width_index(self) -> usize {
        match self {
            Self::R11x27 | Self::R13x27 => 0,
            Self::R7x43
            | Self::R9x43
            | Self::R11x43
            | Self::R13x43
            | Self::R15x43
            | Self::R17x43 => 1,
            Self::R7x59
            | Self::R9x59
            | Self::R11x59
            | Self::R13x59
            | Self::R15x59
            | Self::R17x59 => 2,
            Self::R7x77
            | Self::R9x77
            | Self::R11x77
            | Self::R13x77
            | Self::R15x77
            | Self::R17x77 => 3,
            Self::R7x99
            | Self::R9x99
            | Self::R11x99
            | Self::R13x99
            | Self::R15x99
            | Self::R17x99 => 4,
            Self::R7x139
            | Self::R9x139
            | Self::R11x139
            | Self::R13x139
            | Self::R15x139
            | Self::R17x139 => 5,
        }
    }
}

impl From<RectMicroVersion> for (u8, u8) {
    /// Converts a `RectMicroVersion` into a `(height, width)` tuple.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::RectMicroVersion;
    ///
    /// assert_eq!(<(u8, u8)>::from(RectMicroVersion::R7x43), (7, 43));
    /// assert_eq!(<(u8, u8)>::from(RectMicroVersion::R17x139), (17, 139));
    /// ```
    fn from(version: RectMicroVersion) -> Self {
        match version {
            RectMicroVersion::R7x43 => (7, 43),
            RectMicroVersion::R7x59 => (7, 59),
            RectMicroVersion::R7x77 => (7, 77),
            RectMicroVersion::R7x99 => (7, 99),
            RectMicroVersion::R7x139 => (7, 139),
            RectMicroVersion::R9x43 => (9, 43),
            RectMicroVersion::R9x59 => (9, 59),
            RectMicroVersion::R9x77 => (9, 77),
            RectMicroVersion::R9x99 => (9, 99),
            RectMicroVersion::R9x139 => (9, 139),
            RectMicroVersion::R11x27 => (11, 27),
            RectMicroVersion::R11x43 => (11, 43),
            RectMicroVersion::R11x59 => (11, 59),
            RectMicroVersion::R11x77 => (11, 77),
            RectMicroVersion::R11x99 => (11, 99),
            RectMicroVersion::R11x139 => (11, 139),
            RectMicroVersion::R13x27 => (13, 27),
            RectMicroVersion::R13x43 => (13, 43),
            RectMicroVersion::R13x59 => (13, 59),
            RectMicroVersion::R13x77 => (13, 77),
            RectMicroVersion::R13x99 => (13, 99),
            RectMicroVersion::R13x139 => (13, 139),
            RectMicroVersion::R15x43 => (15, 43),
            RectMicroVersion::R15x59 => (15, 59),
            RectMicroVersion::R15x77 => (15, 77),
            RectMicroVersion::R15x99 => (15, 99),
            RectMicroVersion::R15x139 => (15, 139),
            RectMicroVersion::R17x43 => (17, 43),
            RectMicroVersion::R17x59 => (17, 59),
            RectMicroVersion::R17x77 => (17, 77),
            RectMicroVersion::R17x99 => (17, 99),
            RectMicroVersion::R17x139 => (17, 139),
        }
    }
}

impl TryFrom<(u8, u8)> for RectMicroVersion {
    type Error = Error;

    /// Converts a `(height, width)` tuple to a `RectMicroVersion`.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if `version` is not a valid rMQR code version.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::RectMicroVersion;
    ///
    /// assert_eq!(
    ///     RectMicroVersion::try_from((7, 43)),
    ///     Ok(RectMicroVersion::R7x43)
    /// );
    /// assert_eq!(
    ///     RectMicroVersion::try_from((17, 139)),
    ///     Ok(RectMicroVersion::R17x139)
    /// );
    ///
    /// assert!(RectMicroVersion::try_from((0, 0)).is_err());
    /// ```
    fn try_from(version: (u8, u8)) -> Result<Self, Self::Error> {
        match version {
            (7, 43) => Ok(Self::R7x43),
            (7, 59) => Ok(Self::R7x59),
            (7, 77) => Ok(Self::R7x77),
            (7, 99) => Ok(Self::R7x99),
            (7, 139) => Ok(Self::R7x139),
            (9, 43) => Ok(Self::R9x43),
            (9, 59) => Ok(Self::R9x59),
            (9, 77) => Ok(Self::R9x77),
            (9, 99) => Ok(Self::R9x99),
            (9, 139) => Ok(Self::R9x139),
            (11, 27) => Ok(Self::R11x27),
            (11, 43) => Ok(Self::R11x43),
            (11, 59) => Ok(Self::R11x59),
            (11, 77) => Ok(Self::R11x77),
            (11, 99) => Ok(Self::R11x99),
            (11, 139) => Ok(Self::R11x139),
            (13, 27) => Ok(Self::R13x27),
            (13, 43) => Ok(Self::R13x43),
            (13, 59) => Ok(Self::R13x59),
            (13, 77) => Ok(Self::R13x77),
            (13, 99) => Ok(Self::R13x99),
            (13, 139) => Ok(Self::R13x139),
            (15, 43) => Ok(Self::R15x43),
            (15, 59) => Ok(Self::R15x59),
            (15, 77) => Ok(Self::R15x77),
            (15, 99) => Ok(Self::R15x99),
            (15, 139) => Ok(Self::R15x139),
            (17, 43) => Ok(Self::R17x43),
            (17, 59) => Ok(Self::R17x59),
            (17, 77) => Ok(Self::R17x77),
            (17, 99) => Ok(Self::R17x99),
            (17, 139) => Ok(Self::R17x139),
            _ => Err(Error::InvalidVersion),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_rect_micro_version_to_u8_tuple() {
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R7x43), (7, 43));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R7x59), (7, 59));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R7x77), (7, 77));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R7x99), (7, 99));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R7x139), (7, 139));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R9x43), (9, 43));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R9x59), (9, 59));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R9x77), (9, 77));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R9x99), (9, 99));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R9x139), (9, 139));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R11x27), (11, 27));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R11x43), (11, 43));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R11x59), (11, 59));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R11x77), (11, 77));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R11x99), (11, 99));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R11x139), (11, 139));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R13x27), (13, 27));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R13x43), (13, 43));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R13x59), (13, 59));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R13x77), (13, 77));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R13x99), (13, 99));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R13x139), (13, 139));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R15x43), (15, 43));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R15x59), (15, 59));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R15x77), (15, 77));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R15x99), (15, 99));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R15x139), (15, 139));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R17x43), (17, 43));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R17x59), (17, 59));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R17x77), (17, 77));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R17x99), (17, 99));
        assert_eq!(<(u8, u8)>::from(RectMicroVersion::R17x139), (17, 139));
    }

    #[test]
    fn try_from_u8_tuple_to_rect_micro_version() {
        assert_eq!(
            RectMicroVersion::try_from((7, 43)).unwrap(),
            RectMicroVersion::R7x43
        );
        assert_eq!(
            RectMicroVersion::try_from((7, 59)).unwrap(),
            RectMicroVersion::R7x59
        );
        assert_eq!(
            RectMicroVersion::try_from((7, 77)).unwrap(),
            RectMicroVersion::R7x77
        );
        assert_eq!(
            RectMicroVersion::try_from((7, 99)).unwrap(),
            RectMicroVersion::R7x99
        );
        assert_eq!(
            RectMicroVersion::try_from((7, 139)).unwrap(),
            RectMicroVersion::R7x139
        );
        assert_eq!(
            RectMicroVersion::try_from((9, 43)).unwrap(),
            RectMicroVersion::R9x43
        );
        assert_eq!(
            RectMicroVersion::try_from((9, 59)).unwrap(),
            RectMicroVersion::R9x59
        );
        assert_eq!(
            RectMicroVersion::try_from((9, 77)).unwrap(),
            RectMicroVersion::R9x77
        );
        assert_eq!(
            RectMicroVersion::try_from((9, 99)).unwrap(),
            RectMicroVersion::R9x99
        );
        assert_eq!(
            RectMicroVersion::try_from((9, 139)).unwrap(),
            RectMicroVersion::R9x139
        );
        assert_eq!(
            RectMicroVersion::try_from((11, 27)).unwrap(),
            RectMicroVersion::R11x27
        );
        assert_eq!(
            RectMicroVersion::try_from((11, 43)).unwrap(),
            RectMicroVersion::R11x43
        );
        assert_eq!(
            RectMicroVersion::try_from((11, 59)).unwrap(),
            RectMicroVersion::R11x59
        );
        assert_eq!(
            RectMicroVersion::try_from((11, 77)).unwrap(),
            RectMicroVersion::R11x77
        );
        assert_eq!(
            RectMicroVersion::try_from((11, 99)).unwrap(),
            RectMicroVersion::R11x99
        );
        assert_eq!(
            RectMicroVersion::try_from((11, 139)).unwrap(),
            RectMicroVersion::R11x139
        );
        assert_eq!(
            RectMicroVersion::try_from((13, 27)).unwrap(),
            RectMicroVersion::R13x27
        );
        assert_eq!(
            RectMicroVersion::try_from((13, 43)).unwrap(),
            RectMicroVersion::R13x43
        );
        assert_eq!(
            RectMicroVersion::try_from((13, 59)).unwrap(),
            RectMicroVersion::R13x59
        );
        assert_eq!(
            RectMicroVersion::try_from((13, 77)).unwrap(),
            RectMicroVersion::R13x77
        );
        assert_eq!(
            RectMicroVersion::try_from((13, 99)).unwrap(),
            RectMicroVersion::R13x99
        );
        assert_eq!(
            RectMicroVersion::try_from((13, 139)).unwrap(),
            RectMicroVersion::R13x139
        );
        assert_eq!(
            RectMicroVersion::try_from((15, 43)).unwrap(),
            RectMicroVersion::R15x43
        );
        assert_eq!(
            RectMicroVersion::try_from((15, 59)).unwrap(),
            RectMicroVersion::R15x59
        );
        assert_eq!(
            RectMicroVersion::try_from((15, 77)).unwrap(),
            RectMicroVersion::R15x77
        );
        assert_eq!(
            RectMicroVersion::try_from((15, 99)).unwrap(),
            RectMicroVersion::R15x99
        );
        assert_eq!(
            RectMicroVersion::try_from((15, 139)).unwrap(),
            RectMicroVersion::R15x139
        );
        assert_eq!(
            RectMicroVersion::try_from((17, 43)).unwrap(),
            RectMicroVersion::R17x43
        );
        assert_eq!(
            RectMicroVersion::try_from((17, 59)).unwrap(),
            RectMicroVersion::R17x59
        );
        assert_eq!(
            RectMicroVersion::try_from((17, 77)).unwrap(),
            RectMicroVersion::R17x77
        );
        assert_eq!(
            RectMicroVersion::try_from((17, 99)).unwrap(),
            RectMicroVersion::R17x99
        );
        assert_eq!(
            RectMicroVersion::try_from((17, 139)).unwrap(),
            RectMicroVersion::R17x139
        );
    }

    #[test]
    fn try_from_u8_tuple_to_rect_micro_version_with_invalid_version() {
        assert_eq!(
            RectMicroVersion::try_from((0, 0)).unwrap_err(),
            Error::InvalidVersion
        );
        assert_eq!(
            RectMicroVersion::try_from((u8::MAX, u8::MAX)).unwrap_err(),
            Error::InvalidVersion
        );
    }
}
