// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2019 Ivan Tham
// SPDX-FileCopyrightText: 2020 Riccardo Casatta
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The `bits` module encodes binary data into raw bits used in a QR code.

use alloc::vec::Vec;
use core::cmp;

use crate::{
    cast::{As, Truncate},
    optimize::{self, Optimizer, Parser, Segment},
    types::{EcLevel, Mode, QrError, QrResult, Version},
};

// Bits

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
    /// # use qrcode2::{Version, bits::Bits};
    /// #
    /// let bits = Bits::new(Version::Normal(1));
    /// ```
    #[must_use]
    #[inline]
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
    pub fn push_number_checked(&mut self, n: usize, number: usize) -> QrResult<()> {
        if n > 16 || number >= (1 << n) {
            Err(QrError::DataTooLong)
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
    /// # use qrcode2::{Version, bits::Bits};
    /// #
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
    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    /// Returns the total number of bits currently pushed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::{Version, bits::Bits};
    /// #
    /// let mut bits = Bits::new(Version::Normal(1));
    /// assert_eq!(bits.len(), 0);
    ///
    /// bits.push_numeric_data(b"01234567");
    /// assert_eq!(bits.len(), 41);
    /// ```
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
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
    /// # use qrcode2::{Version, bits::Bits};
    /// #
    /// let mut bits = Bits::new(Version::Normal(1));
    /// assert_eq!(bits.is_empty(), true);
    ///
    /// bits.push_numeric_data(b"01234567");
    /// assert_eq!(bits.is_empty(), false);
    /// ```
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// The maximum number of bits allowed by the provided QR code version and
    /// error correction level.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if it is not valid to use the `ec_level` for the given
    /// version (e.g. [`Version::Micro(1)`](Version::Micro) with
    /// [`EcLevel::H`]).
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::{EcLevel, Version, bits::Bits};
    /// #
    /// let bits = Bits::new(Version::Normal(1));
    /// assert_eq!(bits.max_len(EcLevel::M), Ok(128));
    /// ```
    #[inline]
    pub fn max_len(&self, ec_level: EcLevel) -> QrResult<usize> {
        self.version.fetch(ec_level, &DATA_LENGTHS)
    }

    /// Returns the version of the QR code.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::{Version, bits::Bits};
    /// #
    /// let bits = Bits::new(Version::Normal(1));
    /// assert_eq!(bits.version(), Version::Normal(1));
    /// ```
    #[must_use]
    #[inline]
    pub const fn version(&self) -> Version {
        self.version
    }
}

#[cfg(test)]
mod push_number_tests {
    use super::*;

    #[test]
    fn test_push_number() {
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

// Mode indicator

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
    pub fn push_mode_indicator(&mut self, mode: ExtendedMode) -> QrResult<()> {
        #[allow(clippy::match_same_arms)]
        let number = match (self.version, mode) {
            (Version::Micro(1), ExtendedMode::Data(Mode::Numeric)) => return Ok(()),
            (Version::Micro(_), ExtendedMode::Data(Mode::Numeric)) => 0,
            (Version::Micro(_), ExtendedMode::Data(Mode::Alphanumeric)) => 1,
            (Version::Micro(_), ExtendedMode::Data(Mode::Byte)) => 0b10,
            (Version::Micro(_), ExtendedMode::Data(Mode::Kanji)) => 0b11,
            (Version::Micro(_), _) => return Err(QrError::UnsupportedCharacterSet),
            (Version::RectMicro(..), ExtendedMode::Data(Mode::Numeric)) => 0b001,
            (Version::RectMicro(..), ExtendedMode::Data(Mode::Alphanumeric)) => 0b010,
            (Version::RectMicro(..), ExtendedMode::Data(Mode::Byte)) => 0b011,
            (Version::RectMicro(..), ExtendedMode::Data(Mode::Kanji)) => 0b100,
            (Version::RectMicro(..), ExtendedMode::Eci) => 0b111,
            (Version::RectMicro(..), ExtendedMode::Fnc1First) => 0b101,
            (Version::RectMicro(..), ExtendedMode::Fnc1Second) => 0b110,
            (Version::RectMicro(..), _) => return Err(QrError::UnsupportedCharacterSet),
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
            .or(Err(QrError::UnsupportedCharacterSet))
    }
}

// ECI

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
    /// bits.push_byte_data(b"\xa1\xa2\xa3\xa4\xa5");
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
                self.push_number(16, (eci_designator & 0xffff).as_u16());
            }
            _ => return Err(QrError::InvalidEciDesignator),
        }
        Ok(())
    }
}

#[cfg(test)]
mod eci_tests {
    use super::*;

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

// `Mode::Numeric` mode

impl Bits {
    fn push_header(&mut self, mode: Mode, raw_data_len: usize) -> QrResult<()> {
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
    pub fn push_numeric_data(&mut self, data: &[u8]) -> QrResult<()> {
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
mod numeric_tests {
    use super::*;

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
        assert_eq!(
            bits.push_numeric_data(b"12345678"),
            Err(QrError::DataTooLong)
        );
    }
}

// `Mode::Alphanumeric` mode

/// In QR code [`Mode::Alphanumeric`] mode, a pair of alphanumeric characters
/// will be encoded as a base-45 integer. `alphanumeric_digit` converts each
/// character into its corresponding base-45 digit.
///
/// The conversion is specified in ISO/IEC 18004:2006, §8.4.3, Table 5.
#[inline]
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
    pub fn push_alphanumeric_data(&mut self, data: &[u8]) -> QrResult<()> {
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
mod alphanumeric_tests {
    use super::*;

    #[test]
    fn test_iso_18004_2006_example() {
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
    fn test_micro_qr_unsupported() {
        let mut bits = Bits::new(Version::Micro(1));
        assert_eq!(
            bits.push_alphanumeric_data(b"A"),
            Err(QrError::UnsupportedCharacterSet)
        );
    }

    #[test]
    fn test_data_too_long() {
        let mut bits = Bits::new(Version::Micro(2));
        assert_eq!(
            bits.push_alphanumeric_data(b"ABCDEFGH"),
            Err(QrError::DataTooLong)
        );
    }
}

// `Mode::Byte` mode

impl Bits {
    /// Encodes 8-bit byte data to the bits.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] on overflow.
    pub fn push_byte_data(&mut self, data: &[u8]) -> QrResult<()> {
        self.push_header(Mode::Byte, data.len())?;
        for b in data {
            self.push_number(8, u16::from(*b));
        }
        Ok(())
    }
}

#[cfg(test)]
mod byte_tests {
    use super::*;

    #[test]
    fn test() {
        let mut bits = Bits::new(Version::Normal(1));
        assert_eq!(
            bits.push_byte_data(b"\x12\x34\x56\x78\x9a\xbc\xde\xf0"),
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
            Err(QrError::UnsupportedCharacterSet)
        );
    }

    #[test]
    fn test_data_too_long() {
        let mut bits = Bits::new(Version::Micro(3));
        assert_eq!(
            bits.push_byte_data(b"0123456701234567"),
            Err(QrError::DataTooLong)
        );
    }
}

// `Mode::Kanji` mode

impl Bits {
    /// Encodes Shift JIS double-byte data to the bits.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] on overflow, or if the data is not Shift JIS double-byte
    /// data (e.g. if the length of data is not an even number).
    pub fn push_kanji_data(&mut self, data: &[u8]) -> QrResult<()> {
        self.push_header(Mode::Kanji, data.len() / 2)?;
        for kanji in data.chunks(2) {
            if kanji.len() != 2 {
                return Err(QrError::InvalidCharacter);
            }
            let cp = u16::from(kanji[0]) * 256 + u16::from(kanji[1]);
            let bytes = if cp < 0xe040 {
                cp - 0x8140
            } else {
                cp - 0xc140
            };
            let number = (bytes >> 8) * 0xc0 + (bytes & 0xff);
            self.push_number(13, number);
        }
        Ok(())
    }
}

#[cfg(test)]
mod kanji_tests {
    use super::*;

    #[test]
    fn test_iso_18004_example() {
        let mut bits = Bits::new(Version::Normal(1));
        assert_eq!(bits.push_kanji_data(b"\x93\x5f\xe4\xaa"), Ok(()));
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
    fn test_micro_qr_unsupported() {
        let mut bits = Bits::new(Version::Micro(2));
        assert_eq!(
            bits.push_kanji_data(b"?"),
            Err(QrError::UnsupportedCharacterSet)
        );
    }

    #[test]
    fn test_data_too_long() {
        let mut bits = Bits::new(Version::Micro(3));
        assert_eq!(
            bits.push_kanji_data(b"\x93_\x93_\x93_\x93_\x93_\x93_\x93_\x93_"),
            Err(QrError::DataTooLong)
        );
    }
}

// FNC1 mode

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
    /// # use qrcode2::{Version, bits::Bits};
    /// #
    /// let mut bits = Bits::new(Version::Normal(1));
    /// bits.push_fnc1_first_position();
    /// bits.push_numeric_data(b"01049123451234591597033130128");
    /// bits.push_alphanumeric_data(b"%10ABC123");
    /// ```
    #[inline]
    pub fn push_fnc1_first_position(&mut self) -> QrResult<()> {
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
    /// # use qrcode2::{Version, bits::Bits};
    /// #
    /// let mut bits = Bits::new(Version::Normal(1));
    /// bits.push_fnc1_second_position(37);
    /// bits.push_alphanumeric_data(b"AA1234BBB112");
    /// bits.push_byte_data(b"text text text text\r");
    /// ```
    ///
    /// If the application indicator is a single Latin alphabet (a–z / A–Z),
    /// please pass in its ASCII value + 100:
    ///
    /// ```
    /// # use qrcode2::{Version, bits::Bits};
    /// #
    /// let mut bits = Bits::new(Version::Normal(1));
    /// bits.push_fnc1_second_position(b'A' + 100);
    /// ```
    pub fn push_fnc1_second_position(&mut self, application_indicator: u8) -> QrResult<()> {
        self.push_mode_indicator(ExtendedMode::Fnc1Second)?;
        self.push_number(8, u16::from(application_indicator));
        Ok(())
    }
}

// Finish

// This table is copied from ISO/IEC 18004:2006 §6.4.10, Table 7, and ISO/IEC
// 23941:2022 Table 6.
static DATA_LENGTHS: [[usize; 4]; 76] = [
    // Normal versions
    [152, 128, 104, 72],
    [272, 224, 176, 128],
    [440, 352, 272, 208],
    [640, 512, 384, 288],
    [864, 688, 496, 368],
    [1088, 864, 608, 480],
    [1248, 992, 704, 528],
    [1552, 1232, 880, 688],
    [1856, 1456, 1056, 800],
    [2192, 1728, 1232, 976],
    [2592, 2032, 1440, 1120],
    [2960, 2320, 1648, 1264],
    [3424, 2672, 1952, 1440],
    [3688, 2920, 2088, 1576],
    [4184, 3320, 2360, 1784],
    [4712, 3624, 2600, 2024],
    [5176, 4056, 2936, 2264],
    [5768, 4504, 3176, 2504],
    [6360, 5016, 3560, 2728],
    [6888, 5352, 3880, 3080],
    [7456, 5712, 4096, 3248],
    [8048, 6256, 4544, 3536],
    [8752, 6880, 4912, 3712],
    [9392, 7312, 5312, 4112],
    [10208, 8000, 5744, 4304],
    [10960, 8496, 6032, 4768],
    [11744, 9024, 6464, 5024],
    [12248, 9544, 6968, 5288],
    [13048, 10136, 7288, 5608],
    [13880, 10984, 7880, 5960],
    [14744, 11640, 8264, 6344],
    [15640, 12328, 8920, 6760],
    [16568, 13048, 9368, 7208],
    [17528, 13800, 9848, 7688],
    [18448, 14496, 10288, 7888],
    [19472, 15312, 10832, 8432],
    [20528, 15936, 11408, 8768],
    [21616, 16816, 12016, 9136],
    [22496, 17728, 12656, 9776],
    [23648, 18672, 13328, 10208],
    // Micro versions
    [20, 0, 0, 0],
    [40, 32, 0, 0],
    [84, 68, 0, 0],
    [128, 112, 80, 0],
    // rMQR versions
    [0, 48, 0, 24],
    [0, 96, 0, 56],
    [0, 160, 0, 80],
    [0, 224, 0, 112],
    [0, 352, 0, 192],
    [0, 96, 0, 56],
    [0, 168, 0, 88],
    [0, 248, 0, 136],
    [0, 336, 0, 176],
    [0, 504, 0, 264],
    [0, 56, 0, 40],
    [0, 152, 0, 88],
    [0, 248, 0, 120],
    [0, 344, 0, 184],
    [0, 456, 0, 232],
    [0, 672, 0, 336],
    [0, 96, 0, 56],
    [0, 216, 0, 104],
    [0, 304, 0, 160],
    [0, 424, 0, 232],
    [0, 584, 0, 280],
    [0, 848, 0, 432],
    [0, 264, 0, 120],
    [0, 384, 0, 208],
    [0, 536, 0, 248],
    [0, 704, 0, 384],
    [0, 1016, 0, 552],
    [0, 312, 0, 168],
    [0, 448, 0, 224],
    [0, 624, 0, 304],
    [0, 800, 0, 448],
    [0, 1216, 0, 608],
];

impl Bits {
    /// Pushes the ending bits to indicate no more data.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] on overflow, or if it is not valid to use the `ec_level`
    /// for the given version (e.g. [`Version::Micro(1)`](Version::Micro) with
    /// [`EcLevel::H`]).
    pub fn push_terminator(&mut self, ec_level: EcLevel) -> QrResult<()> {
        let terminator_size = match self.version {
            Version::Micro(a) => a.as_usize() * 2 + 1,
            Version::RectMicro(..) => 3,
            Version::Normal(_) => 4,
        };

        let cur_length = self.len();
        let data_length = self.max_len(ec_level)?;
        if cur_length > data_length {
            return Err(QrError::DataTooLong);
        }

        let terminator_size = cmp::min(terminator_size, data_length - cur_length);
        if terminator_size > 0 {
            self.push_number(terminator_size, 0);
        }

        if self.len() < data_length {
            const PADDING_BYTES: [u8; 2] = [0b1110_1100, 0b0001_0001];

            self.bit_offset = 0;
            let data_bytes_length = data_length / 8;
            let padding_bytes_count = data_bytes_length.saturating_sub(self.data.len());
            let padding = PADDING_BYTES
                .iter()
                .copied()
                .cycle()
                .take(padding_bytes_count);
            self.data.extend(padding);
        }

        if self.len() < data_length {
            self.data.push(0);
        }

        Ok(())
    }
}

#[cfg(test)]
mod finish_tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let mut bits = Bits::new(Version::Normal(1));
        assert_eq!(bits.push_alphanumeric_data(b"HELLO WORLD"), Ok(()));
        assert_eq!(bits.push_terminator(EcLevel::Q), Ok(()));
        assert_eq!(
            bits.into_bytes(),
            [
                0b0010_0000,
                0b0101_1011,
                0b0000_1011,
                0b0111_1000,
                0b1101_0001,
                0b0111_0010,
                0b1101_1100,
                0b0100_1101,
                0b0100_0011,
                0b0100_0000,
                0b1110_1100,
                0b0001_0001,
                0b1110_1100,
            ]
        );
    }

    #[test]
    fn test_too_long() {
        let mut bits = Bits::new(Version::Micro(1));
        assert_eq!(bits.push_numeric_data(b"9999999"), Ok(()));
        assert_eq!(bits.push_terminator(EcLevel::L), Err(QrError::DataTooLong));
    }

    #[test]
    fn test_no_terminator() {
        let mut bits = Bits::new(Version::Micro(1));
        assert_eq!(bits.push_numeric_data(b"99999"), Ok(()));
        assert_eq!(bits.push_terminator(EcLevel::L), Ok(()));
        assert_eq!(bits.into_bytes(), [0b1011_1111, 0b0011_1110, 0b0011_0000]);
    }

    #[test]
    fn test_no_padding() {
        let mut bits = Bits::new(Version::Micro(1));
        assert_eq!(bits.push_numeric_data(b"9999"), Ok(()));
        assert_eq!(bits.push_terminator(EcLevel::L), Ok(()));
        assert_eq!(bits.into_bytes(), [0b1001_1111, 0b0011_1100, 0b1000_0000]);
    }

    #[test]
    fn test_micro_version_1_half_byte_padding() {
        let mut bits = Bits::new(Version::Micro(1));
        assert_eq!(bits.push_numeric_data(b"999"), Ok(()));
        assert_eq!(bits.push_terminator(EcLevel::L), Ok(()));
        assert_eq!(bits.into_bytes(), [0b0111_1111, 0b0011_1000, 0b0000_0000]);
    }

    #[test]
    fn test_micro_version_1_full_byte_padding() {
        let mut bits = Bits::new(Version::Micro(1));
        assert_eq!(bits.push_numeric_data(b""), Ok(()));
        assert_eq!(bits.push_terminator(EcLevel::L), Ok(()));
        assert_eq!(bits.into_bytes(), [0b0000_0000, 0b1110_1100, 0]);
    }
}

// Front end

impl Bits {
    /// Pushes a segmented data to the bits, and then terminate it.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] on overflow, or if the segment refers to incorrectly
    /// encoded byte sequence.
    pub fn push_segments<I>(&mut self, data: &[u8], segments_iter: I) -> QrResult<()>
    where
        I: Iterator<Item = Segment>,
    {
        for segment in segments_iter {
            let slice = &data[segment.begin..segment.end];
            match segment.mode {
                Mode::Numeric => self.push_numeric_data(slice),
                Mode::Alphanumeric => self.push_alphanumeric_data(slice),
                Mode::Byte => self.push_byte_data(slice),
                Mode::Kanji => self.push_kanji_data(slice),
            }?;
        }
        Ok(())
    }

    /// Pushes the data the bits, using the optimal encoding.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] on overflow.
    #[inline]
    pub fn push_optimal_data(&mut self, data: &[u8]) -> QrResult<()> {
        let segments = Parser::new(data).optimize(self.version);
        self.push_segments(data, segments)
    }
}

#[cfg(test)]
mod encode_tests {
    use alloc::vec;

    use super::*;

    fn encode(data: &[u8], version: Version, ec_level: EcLevel) -> QrResult<Vec<u8>> {
        let mut bits = Bits::new(version);
        bits.push_optimal_data(data)?;
        bits.push_terminator(ec_level)?;
        Ok(bits.into_bytes())
    }

    #[test]
    fn test_alphanumeric() {
        let res = encode(b"HELLO WORLD", Version::Normal(1), EcLevel::Q);
        assert_eq!(
            res,
            Ok(vec![
                0b0010_0000,
                0b0101_1011,
                0b0000_1011,
                0b0111_1000,
                0b1101_0001,
                0b0111_0010,
                0b1101_1100,
                0b0100_1101,
                0b0100_0011,
                0b0100_0000,
                0b1110_1100,
                0b0001_0001,
                0b1110_1100,
            ])
        );
    }

    #[test]
    fn test_auto_mode_switch() {
        let res = encode(b"123A", Version::Micro(2), EcLevel::L);
        assert_eq!(
            res,
            Ok(vec![
                0b0001_1000,
                0b1111_0111,
                0b0010_0101,
                0b0000_0000,
                0b1110_1100
            ])
        );
    }

    #[test]
    fn test_too_long() {
        let res = encode(b">>>>>>>>", Version::Normal(1), EcLevel::H);
        assert_eq!(res, Err(QrError::DataTooLong));
    }
}

// Auto version minimization

#[allow(clippy::missing_panics_doc)]
/// Automatically determines the minimum QR code version to store the data, and
/// encode the result.
///
/// This method will not consider any Micro QR code or rMQR code versions.
///
/// # Errors
///
/// Returns [`Err`] if the data is too long to fit even the highest QR code
/// version.
///
/// # Examples
///
/// ```
/// # use qrcode2::{EcLevel, Version, bits};
/// #
/// let bits = bits::encode_auto(b"Hello, world!", EcLevel::M).unwrap();
/// assert_eq!(bits.version(), Version::Normal(1));
/// ```
pub fn encode_auto(data: &[u8], ec_level: EcLevel) -> QrResult<Bits> {
    let segments = Parser::new(data).collect::<Vec<Segment>>();
    for version in &[Version::Normal(9), Version::Normal(26), Version::Normal(40)] {
        let opt_segments = Optimizer::new(segments.iter().copied(), *version).collect::<Vec<_>>();
        let total_len = optimize::total_encoded_len(&opt_segments, *version);
        let data_capacity = version
            .fetch(ec_level, &DATA_LENGTHS)
            .expect("invalid `DATA_LENGTHS`");
        if total_len <= data_capacity {
            let min_version = find_min_version(total_len, ec_level);
            let mut bits = Bits::new(min_version);
            bits.reserve(total_len);
            bits.push_segments(data, opt_segments.into_iter())?;
            bits.push_terminator(ec_level)?;
            return Ok(bits);
        }
    }
    Err(QrError::DataTooLong)
}

/// Finds the smallest version (QR code only) that can store N bits of data in
/// the given error correction level.
fn find_min_version(length: usize, ec_level: EcLevel) -> Version {
    let mut base = 0_usize;
    let mut size = 39;
    while size > 1 {
        let half = size / 2;
        let mid = base + half;
        // mid is always in [0, size).
        // mid >= 0: by definition
        // mid < size: mid = size / 2 + size / 4 + size / 8 ...
        base = if DATA_LENGTHS[mid][ec_level as usize] > length {
            base
        } else {
            mid
        };
        size -= half;
    }
    // base is always in [0, mid) because base <= mid.
    base = if DATA_LENGTHS[base][ec_level as usize] >= length {
        base
    } else {
        base + 1
    };
    Version::Normal((base + 1).as_i16())
}

#[cfg(test)]
mod encode_auto_tests {
    use super::*;

    #[test]
    fn test_find_min_version() {
        assert_eq!(find_min_version(60, EcLevel::L), Version::Normal(1));
        assert_eq!(find_min_version(200, EcLevel::L), Version::Normal(2));
        assert_eq!(find_min_version(200, EcLevel::H), Version::Normal(3));
        assert_eq!(find_min_version(20000, EcLevel::L), Version::Normal(37));
        assert_eq!(find_min_version(640, EcLevel::L), Version::Normal(4));
        assert_eq!(find_min_version(641, EcLevel::L), Version::Normal(5));
        assert_eq!(find_min_version(999_999, EcLevel::H), Version::Normal(40));
    }

    #[test]
    fn test_alpha_q() {
        let bits = encode_auto(b"HELLO WORLD", EcLevel::Q).unwrap();
        assert_eq!(bits.version(), Version::Normal(1));
    }

    #[test]
    fn test_alpha_h() {
        let bits = encode_auto(b"HELLO WORLD", EcLevel::H).unwrap();
        assert_eq!(bits.version(), Version::Normal(2));
    }

    #[test]
    fn test_mixed() {
        let bits = encode_auto(b"This is a mixed data test. 1234567890", EcLevel::H).unwrap();
        assert_eq!(bits.version(), Version::Normal(4));
    }
}

// Auto Micro QR code's version minimization

/// Automatically determines the minimum Micro QR code version to store the
/// data, and encode the result.
///
/// This method will not consider any QR code or rMQR code versions.
///
/// # Errors
///
/// Returns [`Err`] if the data is too long to fit even the highest Micro QR
/// code version.
///
/// # Examples
///
/// ```
/// # use qrcode2::{EcLevel, Version, bits};
/// #
/// let bits = bits::encode_auto_micro(b"Hello, world!", EcLevel::M).unwrap();
/// assert_eq!(bits.version(), Version::Micro(4));
/// ```
pub fn encode_auto_micro(data: &[u8], ec_level: EcLevel) -> QrResult<Bits> {
    let segments = Parser::new(data).collect::<Vec<Segment>>();
    let mut possible_versions = Vec::new();
    for version in 1..=4 {
        let version = Version::Micro(version);
        let opt_segments = Optimizer::new(segments.iter().copied(), version).collect::<Vec<_>>();
        let total_len = optimize::total_encoded_len(&opt_segments, version);
        let data_capacity = version.fetch(ec_level, &DATA_LENGTHS);
        if let Ok(capacity) = data_capacity {
            if total_len <= capacity {
                possible_versions.push(version);
                break;
            }
        }
    }

    let min_version = possible_versions.iter().min_by_key(|v| v.width());

    if let Some(version) = min_version {
        let mut bits = Bits::new(*version);
        let opt_segments = Optimizer::new(segments.iter().copied(), *version).collect::<Vec<_>>();
        bits.reserve(optimize::total_encoded_len(&opt_segments, *version));
        bits.push_segments(data, opt_segments.into_iter())?;
        bits.push_terminator(ec_level)?;
        return Ok(bits);
    }
    Err(QrError::DataTooLong)
}

#[cfg(test)]
mod encode_auto_micro_tests {
    use super::*;

    #[test]
    fn test_alpha_l() {
        let bits = encode_auto_micro(b"HELLO WORLD", EcLevel::L).unwrap();
        assert_eq!(bits.version(), Version::Micro(3));
    }

    #[test]
    fn test_alpha_q() {
        let bits = encode_auto_micro(b"HELLO WORLD", EcLevel::Q).unwrap();
        assert_eq!(bits.version(), Version::Micro(4));
    }

    #[test]
    fn test_mixed() {
        let bits = encode_auto_micro(b"Mixed. 1234567890", EcLevel::M).unwrap();
        assert_eq!(bits.version(), Version::Micro(4));
    }
}

// Auto rMQR code's version minimization

/// Auto rMQR code's version minimization strategy.
#[derive(Clone, Copy, Debug)]
pub enum RectMicroStrategy {
    /// Minimize the width.
    Width,

    /// Minimize the height.
    Height,

    /// Minimize the area.
    Area,
}

/// Automatically determines the minimum rMQR code version to store the data,
/// and encode the result.
///
/// This method will not consider any QR code or Micro QR code versions.
///
/// # Errors
///
/// Returns [`Err`] if the data is too long to fit even the highest rMQR code
/// version.
///
/// # Examples
///
/// ```
/// # use qrcode2::{
/// #     EcLevel, Version,
/// #     bits::{self, RectMicroStrategy},
/// # };
/// #
/// let bits = bits::encode_auto_rect_micro(b"Hello, world!", EcLevel::M, RectMicroStrategy::Area)
///     .unwrap();
/// assert_eq!(bits.version(), Version::RectMicro(11, 43));
/// ```
pub fn encode_auto_rect_micro(
    data: &[u8],
    ec_level: EcLevel,
    strategy: RectMicroStrategy,
) -> QrResult<Bits> {
    let segments = Parser::new(data).collect::<Vec<Segment>>();
    let mut possible_versions = Vec::new();
    for width in Version::RMQR_ALL_WIDTH {
        for height in Version::RMQR_ALL_HEIGHT {
            let version = Version::RectMicro(height, width);
            if !version.is_rect_micro() {
                continue;
            }
            let opt_segments =
                Optimizer::new(segments.iter().copied(), version).collect::<Vec<_>>();
            let total_len = optimize::total_encoded_len(&opt_segments, version);
            let data_capacity = version.fetch(ec_level, &DATA_LENGTHS)?;
            if total_len <= data_capacity {
                possible_versions.push(version);
                break;
            }
        }
    }

    let min_version = match strategy {
        // `possible_versions` is already sorted by width
        RectMicroStrategy::Width => possible_versions.first(),
        RectMicroStrategy::Height => possible_versions.iter().min_by_key(|v| v.height()),
        RectMicroStrategy::Area => possible_versions
            .iter()
            .min_by_key(|v| v.width() * v.height()),
    };

    if let Some(version) = min_version {
        let mut bits = Bits::new(*version);
        let opt_segments = Optimizer::new(segments.iter().copied(), *version).collect::<Vec<_>>();
        bits.reserve(optimize::total_encoded_len(&opt_segments, *version));
        bits.push_segments(data, opt_segments.into_iter())?;
        bits.push_terminator(ec_level)?;
        return Ok(bits);
    }
    Err(QrError::DataTooLong)
}

#[cfg(test)]
mod encode_auto_rect_micro_tests {
    use super::*;

    #[test]
    fn test_alpha_m_width() {
        let bits =
            encode_auto_rect_micro(b"HELLO WORLD", EcLevel::M, RectMicroStrategy::Width).unwrap();
        assert_eq!(bits.version(), Version::RectMicro(13, 27));
    }

    #[test]
    fn test_alpha_m_height() {
        let bits =
            encode_auto_rect_micro(b"HELLO WORLD", EcLevel::M, RectMicroStrategy::Height).unwrap();
        assert_eq!(bits.version(), Version::RectMicro(7, 59));
    }

    #[test]
    fn test_alpha_m_area() {
        let bits =
            encode_auto_rect_micro(b"HELLO WORLD", EcLevel::M, RectMicroStrategy::Area).unwrap();
        assert_eq!(bits.version(), Version::RectMicro(13, 27));
    }

    #[test]
    fn test_alpha_h_width() {
        let bits =
            encode_auto_rect_micro(b"HELLO WORLD", EcLevel::H, RectMicroStrategy::Width).unwrap();
        assert_eq!(bits.version(), Version::RectMicro(11, 43));
    }

    #[test]
    fn test_alpha_h_height() {
        let bits =
            encode_auto_rect_micro(b"HELLO WORLD", EcLevel::H, RectMicroStrategy::Height).unwrap();
        assert_eq!(bits.version(), Version::RectMicro(7, 77));
    }

    #[test]
    fn test_alpha_h_area() {
        let bits =
            encode_auto_rect_micro(b"HELLO WORLD", EcLevel::H, RectMicroStrategy::Area).unwrap();
        assert_eq!(bits.version(), Version::RectMicro(11, 43));
    }

    #[test]
    fn test_mixed_width() {
        let bits = encode_auto_rect_micro(
            b"This is a mixed data test. 1234567890",
            EcLevel::H,
            RectMicroStrategy::Width,
        )
        .unwrap();
        assert_eq!(bits.version(), Version::RectMicro(17, 77));
    }

    #[test]
    fn test_mixed_height() {
        let bits = encode_auto_rect_micro(
            b"This is a mixed data test. 1234567890",
            EcLevel::H,
            RectMicroStrategy::Height,
        )
        .unwrap();
        assert_eq!(bits.version(), Version::RectMicro(11, 139));
    }

    #[test]
    fn test_mixed_area() {
        let bits = encode_auto_rect_micro(
            b"This is a mixed data test. 1234567890",
            EcLevel::H,
            RectMicroStrategy::Area,
        )
        .unwrap();
        assert_eq!(bits.version(), Version::RectMicro(13, 99));
    }
}
