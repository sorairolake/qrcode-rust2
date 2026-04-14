// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of features related to the [`Mode::Numeric`] mode.

use super::{Bits, mode_indicator::ExtendedMode};
use crate::{error::Result, types::Mode};

impl Bits {
    pub(super) fn push_header(&mut self, mode: Mode, raw_data_len: usize) -> Result<()> {
        let length_bits = mode.length_bits_count(self.version);
        self.reserve(length_bits + 4 + mode.data_bits_count(raw_data_len));
        self.push_mode_indicator(ExtendedMode::Data(mode))?;
        self.push_number_checked(length_bits, raw_data_len)?;
        Ok(())
    }

    /// Encodes a numeric string to the bits.
    ///
    /// The data should only contain the characters 0 to 9.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] on overflow.
    pub fn push_numeric_data(&mut self, data: &[u8]) -> Result<()> {
        self.push_header(Mode::Numeric, data.len())?;
        for chunk in data.chunks(3) {
            let number = chunk
                .iter()
                .map(|b| u16::from(*b - b'0'))
                .fold(0, |a, b| a * 10 + b);
            let length = chunk.len() * 3 + 1;
            self.push_number(length, number);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::Error, types::Version};

    #[test]
    fn test_iso_18004_2006_example_1() {
        let mut bits = Bits::new(Version::Normal(1));
        assert_eq!(bits.push_numeric_data(b"01234567"), Ok(()));
        assert_eq!(
            bits.into_bytes(),
            [
                0b0001_0000,
                0b0010_0000,
                0b0000_1100,
                0b0101_0110,
                0b0110_0001,
                0b1000_0000
            ]
        );
    }

    #[test]
    fn test_iso_18004_2000_example_2() {
        let mut bits = Bits::new(Version::Normal(1));
        assert_eq!(bits.push_numeric_data(b"0123456789012345"), Ok(()));
        assert_eq!(
            bits.into_bytes(),
            [
                0b0001_0000,
                0b0100_0000,
                0b0000_1100,
                0b0101_0110,
                0b0110_1010,
                0b0110_1110,
                0b0001_0100,
                0b1110_1010,
                0b0101_0000,
            ]
        );
    }

    #[test]
    fn test_iso_18004_2006_example_2() {
        let mut bits = Bits::new(Version::Micro(3));
        assert_eq!(bits.push_numeric_data(b"0123456789012345"), Ok(()));
        assert_eq!(
            bits.into_bytes(),
            [
                0b0010_0000,
                0b0000_0110,
                0b0010_1011,
                0b0011_0101,
                0b0011_0111,
                0b0000_1010,
                0b0111_0101,
                0b0010_1000,
            ]
        );
    }

    #[test]
    fn test_data_too_long_error() {
        let mut bits = Bits::new(Version::Micro(1));
        assert_eq!(bits.push_numeric_data(b"12345678"), Err(Error::DataTooLong));
    }
}
