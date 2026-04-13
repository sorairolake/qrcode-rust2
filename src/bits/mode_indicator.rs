// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Mode indicator.

use super::Bits;
use crate::{
    error::{Error, Result},
    types::{Mode, Version},
};

/// An "extended" mode indicator, includes all indicators supported by QR code
/// beyond those bearing data.
#[derive(Clone, Copy, Debug)]
pub enum ExtendedMode {
    /// ECI mode indicator, to introduce an ECI designator.
    Eci,

    /// The normal mode to introduce data.
    Data(Mode),

    /// FNC-1 mode in the first position.
    Fnc1First,

    /// FNC-1 mode in the second position.
    Fnc1Second,

    /// Structured append.
    StructuredAppend,
}

impl Bits {
    /// Pushes the mode indicator to the end of the bits.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the mode is not supported in the provided version.
    pub fn push_mode_indicator(&mut self, mode: ExtendedMode) -> Result<()> {
        #[expect(clippy::match_same_arms)]
        let number = match (self.version, mode) {
            (Version::Micro(1), ExtendedMode::Data(Mode::Numeric)) => return Ok(()),
            (Version::Micro(_), ExtendedMode::Data(Mode::Numeric)) => 0,
            (Version::Micro(_), ExtendedMode::Data(Mode::Alphanumeric)) => 1,
            (Version::Micro(_), ExtendedMode::Data(Mode::Byte)) => 0b10,
            (Version::Micro(_), ExtendedMode::Data(Mode::Kanji)) => 0b11,
            (Version::Micro(_), _) => return Err(Error::UnsupportedCharacterSet),
            (Version::RectMicro(..), ExtendedMode::Data(Mode::Numeric)) => 0b001,
            (Version::RectMicro(..), ExtendedMode::Data(Mode::Alphanumeric)) => 0b010,
            (Version::RectMicro(..), ExtendedMode::Data(Mode::Byte)) => 0b011,
            (Version::RectMicro(..), ExtendedMode::Data(Mode::Kanji)) => 0b100,
            (Version::RectMicro(..), ExtendedMode::Eci) => 0b111,
            (Version::RectMicro(..), ExtendedMode::Fnc1First) => 0b101,
            (Version::RectMicro(..), ExtendedMode::Fnc1Second) => 0b110,
            (Version::RectMicro(..), _) => return Err(Error::UnsupportedCharacterSet),
            (_, ExtendedMode::Data(Mode::Numeric)) => 0b0001,
            (_, ExtendedMode::Data(Mode::Alphanumeric)) => 0b0010,
            (_, ExtendedMode::Data(Mode::Byte)) => 0b0100,
            (_, ExtendedMode::Data(Mode::Kanji)) => 0b1000,
            (_, ExtendedMode::Eci) => 0b0111,
            (_, ExtendedMode::Fnc1First) => 0b0101,
            (_, ExtendedMode::Fnc1Second) => 0b1001,
            (_, ExtendedMode::StructuredAppend) => 0b0011,
        };
        let bits = self.version.mode_bits_count();
        self.push_number_checked(bits, number)
            .or(Err(Error::UnsupportedCharacterSet))
    }
}
