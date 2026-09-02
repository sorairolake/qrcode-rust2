// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
// SPDX-FileCopyrightText: 2026 Lars Gerchow
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Encode the data into [`Bits`].

use super::Bits;
use crate::{
    error::Result,
    optimize::{self, Segment},
    types::Mode,
};

impl Bits {
    /// Pushes a segmented data to the bits, and then terminate it.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] on overflow, or if the segment refers to incorrectly
    /// encoded byte sequence.
    pub fn push_segments<I>(&mut self, data: &[u8], segments_iter: I) -> Result<()>
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
    pub fn push_optimal_data(&mut self, data: &[u8]) -> Result<()> {
        let segments = optimize::optimize(data, self.version);
        self.push_segments(data, segments.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::*;
    use crate::{
        error::Error,
        types::{EcLevel, MicroVersion, NormalVersion, Version},
    };

    fn encode(data: &[u8], version: Version, ec_level: EcLevel) -> Result<Vec<u8>> {
        let mut bits = Bits::new(version);
        bits.push_optimal_data(data)?;
        bits.push_terminator(ec_level)?;
        Ok(bits.into_bytes())
    }

    #[test]
    fn alphanumeric() {
        let res = encode(
            b"HELLO WORLD",
            Version::Normal(NormalVersion::V1),
            EcLevel::Q,
        );
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
    fn auto_mode_switch() {
        let res = encode(b"123A", Version::Micro(MicroVersion::M2), EcLevel::L);
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
    fn too_long() {
        let res = encode(b">>>>>>>>", Version::Normal(NormalVersion::V1), EcLevel::H);
        assert_eq!(res, Err(Error::DataTooLong));
    }

    // Regression: 14 QR-alphanumeric characters fit a Micro QR M3-L symbol as a
    // single alphanumeric segment (83 <= 84 bits), but the greedy optimizer
    // used to split the embedded digit run "3935" into its own numeric
    // segment whose extra header overhead overran the capacity, falsely
    // returning DataTooLong.
    #[test]
    fn micro_m3_alphanumeric_with_digit_run() {
        assert!(
            encode(
                b"9BA3935DM3TBE4",
                Version::Micro(MicroVersion::M3),
                EcLevel::L
            )
            .is_ok()
        );
        // One more character genuinely overflows M3 and must still be rejected.
        assert_eq!(
            encode(
                b"9BA3935DM3TBE45",
                Version::Micro(MicroVersion::M3),
                EcLevel::L
            ),
            Err(Error::DataTooLong)
        );
    }
}
