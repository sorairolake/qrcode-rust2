// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of features related to the [`Mode::Alphanumeric`] mode.

use super::Bits;
use crate::{error::Result, types::Mode};

/// In QR code [`Mode::Alphanumeric`] mode, a pair of alphanumeric characters
/// will be encoded as a base-45 integer. `alphanumeric_digit` converts each
/// character into its corresponding base-45 digit.
///
/// The conversion is specified in ISO/IEC 18004:2006, §8.4.3, Table 5.
fn alphanumeric_digit(character: u8) -> u16 {
    match character {
        b'0'..=b'9' => u16::from(character - b'0'),
        b'A'..=b'Z' => u16::from(character - b'A') + 10,
        b' ' => 36,
        b'$' => 37,
        b'%' => 38,
        b'*' => 39,
        b'+' => 40,
        b'-' => 41,
        b'.' => 42,
        b'/' => 43,
        b':' => 44,
        _ => 0,
    }
}

impl Bits {
    /// Encodes an alphanumeric string to the bits.
    ///
    /// The data should only contain the charaters A to Z (excluding lowercase),
    /// 0 to 9, space, `$`, `%`, `*`, `+`, `-`, `.`, `/` or `:`.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] on overflow.
    pub fn push_alphanumeric_data(&mut self, data: &[u8]) -> Result<()> {
        self.push_header(Mode::Alphanumeric, data.len())?;
        for chunk in data.chunks(2) {
            let number = chunk
                .iter()
                .map(|b| alphanumeric_digit(*b))
                .fold(0, |a, b| a * 45 + b);
            let length = chunk.len() * 5 + 1;
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
    fn iso_18004_2006_example() {
        let mut bits = Bits::new(Version::Normal(1));
        assert_eq!(bits.push_alphanumeric_data(b"AC-42"), Ok(()));
        assert_eq!(
            bits.into_bytes(),
            [
                0b0010_0000,
                0b0010_1001,
                0b1100_1110,
                0b1110_0111,
                0b0010_0001,
                0b0000_0000
            ]
        );
    }

    #[test]
    fn micro_qr_unsupported() {
        let mut bits = Bits::new(Version::Micro(1));
        assert_eq!(
            bits.push_alphanumeric_data(b"A"),
            Err(Error::UnsupportedCharacterSet)
        );
    }

    #[test]
    fn data_too_long() {
        let mut bits = Bits::new(Version::Micro(2));
        assert_eq!(
            bits.push_alphanumeric_data(b"ABCDEFGH"),
            Err(Error::DataTooLong)
        );
    }
}
