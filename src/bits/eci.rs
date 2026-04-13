// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ECI.

use super::{Bits, mode_indicator::ExtendedMode};
use crate::{
    cast::As,
    types::{QrError, QrResult},
};

impl Bits {
    /// Pushes an ECI (Extended Channel Interpretation) designator to the bits.
    ///
    /// An ECI designator is a 6-digit number to specify the character set of
    /// the following binary data. After calling this method, one could call
    /// [`Bits::push_byte_data`] or similar methods to insert the actual data.
    ///
    /// The full list of ECI designator values can be found from
    /// <https://en.wikipedia.org/wiki/Extended_Channel_Interpretation>.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the QR code version does not support ECI, or the
    /// designator is outside of the expected range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::{Version, bits::Bits};
    /// #
    /// let mut bits = Bits::new(Version::Normal(1));
    /// // 9 = ISO-8859-7 (Greek).
    /// bits.push_eci_designator(9);
    /// // ΑΒΓΔΕ
    /// bits.push_byte_data(b"\xA1\xA2\xA3\xA4\xA5");
    /// ```
    pub fn push_eci_designator(&mut self, eci_designator: u32) -> QrResult<()> {
        // assume the common case that eci_designator <= 127.
        self.reserve(12);
        self.push_mode_indicator(ExtendedMode::Eci)?;
        match eci_designator {
            0..=127 => {
                self.push_number(8, eci_designator.as_u16());
            }
            128..=16383 => {
                self.push_number(2, 0b10);
                self.push_number(14, eci_designator.as_u16());
            }
            16384..=999_999 => {
                self.push_number(3, 0b110);
                self.push_number(5, (eci_designator >> 16).as_u16());
                self.push_number(16, (eci_designator & 0xFFFF).as_u16());
            }
            _ => return Err(QrError::InvalidEciDesignator),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Version;

    #[test]
    fn test_9() {
        let mut bits = Bits::new(Version::Normal(1));
        assert_eq!(bits.push_eci_designator(9), Ok(()));
        assert_eq!(bits.into_bytes(), [0b0111_0000, 0b1001_0000]);
    }

    #[test]
    fn test_899() {
        let mut bits = Bits::new(Version::Normal(1));
        assert_eq!(bits.push_eci_designator(899), Ok(()));
        assert_eq!(bits.into_bytes(), [0b0111_1000, 0b0011_1000, 0b0011_0000]);
    }

    #[test]
    fn test_999_999() {
        let mut bits = Bits::new(Version::Normal(1));
        assert_eq!(bits.push_eci_designator(999_999), Ok(()));
        assert_eq!(
            bits.into_bytes(),
            [0b0111_1100, 0b1111_0100, 0b0010_0011, 0b1111_0000]
        );
    }

    #[test]
    fn test_invalid_designator() {
        let mut bits = Bits::new(Version::Normal(1));
        assert_eq!(
            bits.push_eci_designator(1_000_000),
            Err(QrError::InvalidEciDesignator)
        );
    }

    #[test]
    fn test_unsupported_character_set() {
        let mut bits = Bits::new(Version::Micro(4));
        assert_eq!(
            bits.push_eci_designator(9),
            Err(QrError::UnsupportedCharacterSet)
        );
    }
}
