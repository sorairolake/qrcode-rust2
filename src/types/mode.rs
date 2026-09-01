// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`Mode`].

use core::cmp::Ordering;

use super::Version;
use crate::cast::As;

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
    /// use qrcode2::{NormalVersion, Version, types::Mode};
    ///
    /// assert_eq!(
    ///     Mode::Numeric.length_bits_count(Version::Normal(NormalVersion::V1)),
    ///     10
    /// );
    /// ```
    #[must_use]
    pub fn length_bits_count(self, version: Version) -> usize {
        match version {
            Version::Micro(a) => {
                let a = u8::from(a).as_usize();
                match self {
                    Self::Numeric => 2 + a,
                    Self::Alphanumeric | Self::Byte => 1 + a,
                    Self::Kanji => a,
                }
            }
            Version::Normal(a) => {
                if a <= NormalVersion::V9 {
                    match self {
                        Self::Numeric => 10,
                        Self::Alphanumeric => 9,
                        Self::Byte | Self::Kanji => 8,
                    }
                } else if a <= NormalVersion::V26 {
                    match self {
                        Self::Numeric => 12,
                        Self::Alphanumeric => 11,
                        Self::Byte => 16,
                        Self::Kanji => 10,
                    }
                } else {
                    match self {
                        Self::Numeric => 14,
                        Self::Alphanumeric => 13,
                        Self::Byte => 16,
                        Self::Kanji => 12,
                    }
                }
            }
            Version::RectMicro(a) => {
                let index = a.index();
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
    /// use qrcode2::types::Mode;
    ///
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
    /// use qrcode2::types::Mode;
    ///
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_order() {
        assert_eq!(
            Mode::Numeric.partial_cmp(&Mode::Numeric).unwrap(),
            Ordering::Equal
        );
        assert_eq!(
            Mode::Alphanumeric.partial_cmp(&Mode::Alphanumeric).unwrap(),
            Ordering::Equal
        );
        assert_eq!(
            Mode::Byte.partial_cmp(&Mode::Byte).unwrap(),
            Ordering::Equal
        );
        assert_eq!(
            Mode::Kanji.partial_cmp(&Mode::Kanji).unwrap(),
            Ordering::Equal
        );

        assert!(Mode::Numeric < Mode::Alphanumeric);
        assert!(Mode::Numeric < Mode::Byte);
        assert!(Mode::Alphanumeric < Mode::Byte);
        assert!(Mode::Kanji < Mode::Byte);

        assert!(Mode::Alphanumeric > Mode::Numeric);
        assert!(Mode::Byte > Mode::Numeric);
        assert!(Mode::Byte > Mode::Alphanumeric);
        assert!(Mode::Byte > Mode::Kanji);

        assert!(Mode::Numeric.partial_cmp(&Mode::Kanji).is_none());
        assert!(Mode::Alphanumeric.partial_cmp(&Mode::Kanji).is_none());
        assert!(Mode::Kanji.partial_cmp(&Mode::Numeric).is_none());
        assert!(Mode::Kanji.partial_cmp(&Mode::Alphanumeric).is_none());
    }

    #[test]
    fn max() {
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
