// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! [`Mode::Byte`] mode.

use super::Bits;
use crate::{error::Result, types::Mode};

impl Bits {
    /// Encodes 8-bit byte data to the bits.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] on overflow.
    pub fn push_byte_data(&mut self, data: &[u8]) -> Result<()> {
        self.push_header(Mode::Byte, data.len())?;
        for b in data {
            self.push_number(8, u16::from(*b));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::Error, types::Version};

    #[test]
    fn test() {
        let mut bits = Bits::new(Version::Normal(1));
        assert_eq!(
            bits.push_byte_data(b"\x12\x34\x56\x78\x9A\xBC\xDE\xF0"),
            Ok(())
        );
        assert_eq!(
            bits.into_bytes(),
            [
                0b0100_0000,
                0b1000_0001,
                0b0010_0011,
                0b0100_0101,
                0b0110_0111,
                0b1000_1001,
                0b1010_1011,
                0b1100_1101,
                0b1110_1111,
                0b0000_0000,
            ]
        );
    }

    #[test]
    fn test_micro_qr_unsupported() {
        let mut bits = Bits::new(Version::Micro(2));
        assert_eq!(
            bits.push_byte_data(b"?"),
            Err(Error::UnsupportedCharacterSet)
        );
    }

    #[test]
    fn test_data_too_long() {
        let mut bits = Bits::new(Version::Micro(3));
        assert_eq!(
            bits.push_byte_data(b"0123456701234567"),
            Err(Error::DataTooLong)
        );
    }
}
