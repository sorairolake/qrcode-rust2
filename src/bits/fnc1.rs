// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of features related to the FNC1 mode.

use super::{Bits, mode_indicator::ExtendedMode};
use crate::error::Result;

impl Bits {
    /// Encodes an indicator that the following data are formatted according to
    /// the UCC/EAN Application Identifiers standard.
    ///
    /// In QR code, the character `%` is used as the data field separator
    /// (0x1D).
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the mode is not supported in the provided version.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{NormalVersion, Version, bits::Bits};
    ///
    /// let mut bits = Bits::new(Version::Normal(NormalVersion::V1));
    /// bits.push_fnc1_first_position();
    /// bits.push_numeric_data(b"01049123451234591597033130128");
    /// bits.push_alphanumeric_data(b"%10ABC123");
    /// ```
    pub fn push_fnc1_first_position(&mut self) -> Result<()> {
        self.push_mode_indicator(ExtendedMode::Fnc1First)
    }

    /// Encodes an indicator that the following data are formatted in accordance
    /// with specific industry or application specifications previously agreed
    /// with AIM International.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the mode is not supported in the provided version.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{NormalVersion, Version, bits::Bits};
    ///
    /// let mut bits = Bits::new(Version::Normal(NormalVersion::V1));
    /// bits.push_fnc1_second_position(37);
    /// bits.push_alphanumeric_data(b"AA1234BBB112");
    /// bits.push_byte_data(b"text text text text\r");
    /// ```
    ///
    /// If the application indicator is a single Latin alphabet (a–z / A–Z),
    /// please pass in its ASCII value + 100:
    ///
    /// ```
    /// use qrcode2::{NormalVersion, Version, bits::Bits};
    ///
    /// let mut bits = Bits::new(Version::Normal(NormalVersion::V1));
    /// bits.push_fnc1_second_position(b'A' + 100);
    /// ```
    pub fn push_fnc1_second_position(&mut self, application_indicator: u8) -> Result<()> {
        self.push_mode_indicator(ExtendedMode::Fnc1Second)?;
        self.push_number(8, u16::from(application_indicator));
        Ok(())
    }
}
