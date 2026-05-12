// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of features related to the [`Mode::Kanji`] mode.

use super::Bits;
use crate::{
    error::{Error, Result},
    types::Mode,
};

impl Bits {
    /// Encodes Shift JIS double-byte data to the bits.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] on overflow, or if the data is not Shift JIS double-byte
    /// data (e.g. if the length of data is not an even number).
    pub fn push_kanji_data(&mut self, data: &[u8]) -> Result<()> {
        self.push_header(Mode::Kanji, data.len() / 2)?;
        for kanji in data.chunks(2) {
            if kanji.len() != 2 {
                return Err(Error::InvalidCharacter);
            }
            let cp = u16::from(kanji[0]) * 256 + u16::from(kanji[1]);
            let bytes = if cp < 0xE040 {
                cp - 0x8140
            } else {
                cp - 0xC140
            };
            let number = (bytes >> 8) * 0xC0 + (bytes & 0xFF);
            self.push_number(13, number);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Version;

    #[test]
    fn iso_18004_example() {
        let mut bits = Bits::new(Version::Normal(1));
        assert_eq!(bits.push_kanji_data(b"\x93\x5F\xE4\xAA"), Ok(()));
        assert_eq!(
            bits.into_bytes(),
            [
                0b1000_0000,
                0b0010_0110,
                0b1100_1111,
                0b1110_1010,
                0b1010_1000
            ]
        );
    }

    #[test]
    fn micro_qr_unsupported() {
        let mut bits = Bits::new(Version::Micro(2));
        assert_eq!(
            bits.push_kanji_data(b"?"),
            Err(Error::UnsupportedCharacterSet)
        );
    }

    #[test]
    fn data_too_long() {
        let mut bits = Bits::new(Version::Micro(3));
        assert_eq!(
            bits.push_kanji_data(b"\x93_\x93_\x93_\x93_\x93_\x93_\x93_\x93_"),
            Err(Error::DataTooLong)
        );
    }
}
