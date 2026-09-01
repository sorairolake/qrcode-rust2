// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2020 Riccardo Casatta
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The `bits` module encodes binary data into raw bits used in a QR code.

mod alphanumeric;
mod auto;
mod byte;
mod eci;
mod encode;
mod fnc1;
mod kanji;
mod mode_indicator;
mod numeric;
mod terminator;

use alloc::vec::Vec;

use self::terminator::DATA_LENGTHS;
pub use self::{
    auto::{RectMicroStrategy, encode_auto, encode_auto_micro, encode_auto_rect_micro},
    mode_indicator::ExtendedMode,
};
use crate::{
    cast::{As, Truncate},
    error::{Error, Result},
    types::{EcLevel, Version},
};

/// The `Bits` structure stores the encoded data for a QR code.
#[derive(Debug)]
pub struct Bits {
    data: Vec<u8>,
    bit_offset: usize,
    version: Version,
}

impl Bits {
    /// Constructs a new, empty bits structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{Version, bits::Bits};
    ///
    /// let bits = Bits::new(Version::Normal(1));
    /// ```
    #[must_use]
    pub const fn new(version: Version) -> Self {
        Self {
            data: Vec::new(),
            bit_offset: 0,
            version,
        }
    }

    /// Pushes an N-bit big-endian integer to the end of the bits.
    ///
    /// <div class="warning">
    ///
    /// It is up to the developer to ensure that `number` really only is `n` bit
    /// in size. Otherwise the excess bits may stomp on the existing ones.
    ///
    /// </div>
    fn push_number(&mut self, n: usize, number: u16) {
        debug_assert!(
            n == 16 || n < 16 && number < (1 << n),
            "{number} is too big as a {n}-bit number"
        );

        let b = self.bit_offset + n;
        let last_index = self.data.len().wrapping_sub(1);
        match (self.bit_offset, b) {
            (0, 0..=8) => {
                self.data.push((number << (8 - b)).truncate_as_u8());
            }
            (0, _) => {
                self.data.push((number >> (b - 8)).truncate_as_u8());
                self.data.push((number << (16 - b)).truncate_as_u8());
            }
            (_, 0..=8) => {
                self.data[last_index] |= (number << (8 - b)).truncate_as_u8();
            }
            (_, 9..=16) => {
                self.data[last_index] |= (number >> (b - 8)).truncate_as_u8();
                self.data.push((number << (16 - b)).truncate_as_u8());
            }
            _ => {
                self.data[last_index] |= (number >> (b - 8)).truncate_as_u8();
                self.data.push((number >> (b - 16)).truncate_as_u8());
                self.data.push((number << (24 - b)).truncate_as_u8());
            }
        }
        self.bit_offset = b & 7;
    }

    /// Pushes an N-bit big-endian integer to the end of the bits, and check
    /// that the number does not overflow the bits.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] on overflow.
    pub fn push_number_checked(&mut self, n: usize, number: usize) -> Result<()> {
        if n > 16 || number >= (1 << n) {
            Err(Error::DataTooLong)
        } else {
            self.push_number(n, number.as_u16());
            Ok(())
        }
    }

    /// Reserves `n` extra bits of space for pushing.
    fn reserve(&mut self, n: usize) {
        let extra_bytes = (n + (8 - self.bit_offset) % 8) / 8;
        self.data.reserve(extra_bytes);
    }

    /// Converts the bits into a bytes vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{Version, bits::Bits};
    ///
    /// let mut bits = Bits::new(Version::Normal(1));
    /// bits.push_numeric_data(b"01234567");
    /// assert_eq!(
    ///     bits.into_bytes(),
    ///     [
    ///         0b0001_0000,
    ///         0b0010_0000,
    ///         0b0000_1100,
    ///         0b0101_0110,
    ///         0b0110_0001,
    ///         0b1000_0000
    ///     ]
    /// );
    /// ```
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    /// Returns the total number of bits currently pushed.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{Version, bits::Bits};
    ///
    /// let mut bits = Bits::new(Version::Normal(1));
    /// assert_eq!(bits.len(), 0);
    ///
    /// bits.push_numeric_data(b"01234567");
    /// assert_eq!(bits.len(), 41);
    /// ```
    #[must_use]
    pub const fn len(&self) -> usize {
        if self.bit_offset == 0 {
            self.data.len() * 8
        } else {
            (self.data.len() - 1) * 8 + self.bit_offset
        }
    }

    /// Returns [`true`] if any bits are not pushed.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{Version, bits::Bits};
    ///
    /// let mut bits = Bits::new(Version::Normal(1));
    /// assert_eq!(bits.is_empty(), true);
    ///
    /// bits.push_numeric_data(b"01234567");
    /// assert_eq!(bits.is_empty(), false);
    /// ```
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// The maximum number of bits allowed by the provided QR code version and
    /// error correction level.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if it is not valid to use the `ec_level` for the given
    /// version.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{EcLevel, Version, bits::Bits};
    ///
    /// let bits = Bits::new(Version::Normal(1));
    /// assert_eq!(bits.max_len(EcLevel::M), Ok(128));
    /// ```
    pub fn max_len(&self, ec_level: EcLevel) -> Result<usize> {
        self.version.fetch(ec_level, &DATA_LENGTHS)
    }

    /// Returns the version of the QR code.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{Version, bits::Bits};
    ///
    /// let bits = Bits::new(Version::Normal(1));
    /// assert_eq!(bits.version(), Version::Normal(1));
    /// ```
    #[must_use]
    pub const fn version(&self) -> Version {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_number() {
        let mut bits = Bits::new(Version::Normal(1));

        // 0:0 .. 0:3
        bits.push_number(3, 0b010);
        // 0:3 .. 0:6
        bits.push_number(3, 0b110);
        // 0:6 .. 1:1
        bits.push_number(3, 0b101);
        // 1:1 .. 2:0
        bits.push_number(7, 0b001_1010);
        // 2:0 .. 2:4
        bits.push_number(4, 0b1100);
        // 2:4 .. 4:0
        bits.push_number(12, 0b1011_0110_1101);
        // 4:0 .. 5:2
        bits.push_number(10, 0b01_1001_0001);
        // 5:2 .. 7:1
        bits.push_number(15, 0b111_0010_1110_0011);

        let bytes = bits.into_bytes();

        assert_eq!(
            bytes,
            [
                // 90
                0b0101_1010,
                // 154
                0b1001_1010,
                // 203
                0b1100_1011,
                // 109
                0b0110_1101,
                // 100
                0b0110_0100,
                // 121
                0b0111_1001,
                // 113
                0b0111_0001,
                // 128
                0b1000_0000,
            ]
        );
    }
}
