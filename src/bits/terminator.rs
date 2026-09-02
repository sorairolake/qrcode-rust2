// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of features related to the terminator.

use core::cmp;

use super::Bits;
use crate::{
    cast::As,
    error::{Error, Result},
    types::{EcLevel, Version},
};

/// This table is copied from ISO/IEC 18004:2006 §6.4.10, Table 7, and ISO/IEC
/// 23941:2022 Table 6.
pub static DATA_LENGTHS: [[usize; 4]; 76] = [
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
    /// for the given version.
    pub fn push_terminator(&mut self, ec_level: EcLevel) -> Result<()> {
        let terminator_size = match self.version {
            Version::Micro(a) => a.as_usize() * 2 + 1,
            Version::RectMicro(..) => 3,
            Version::Normal(_) => 4,
        };

        let cur_length = self.len();
        let data_length = self.max_len(ec_level)?;
        if cur_length > data_length {
            return Err(Error::DataTooLong);
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
mod tests {
    use super::*;

    #[test]
    fn hello_world() {
        let mut bits = Bits::new(Version::Normal(NormalVersion::V1));
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
    fn too_long() {
        let mut bits = Bits::new(Version::Micro(MicroVersion::M1));
        assert_eq!(bits.push_numeric_data(b"9999999"), Ok(()));
        assert_eq!(bits.push_terminator(EcLevel::L), Err(Error::DataTooLong));
    }

    #[test]
    fn no_terminator() {
        let mut bits = Bits::new(Version::Micro(MicroVersion::M1));
        assert_eq!(bits.push_numeric_data(b"99999"), Ok(()));
        assert_eq!(bits.push_terminator(EcLevel::L), Ok(()));
        assert_eq!(bits.into_bytes(), [0b1011_1111, 0b0011_1110, 0b0011_0000]);
    }

    #[test]
    fn no_padding() {
        let mut bits = Bits::new(Version::Micro(MicroVersion::M1));
        assert_eq!(bits.push_numeric_data(b"9999"), Ok(()));
        assert_eq!(bits.push_terminator(EcLevel::L), Ok(()));
        assert_eq!(bits.into_bytes(), [0b1001_1111, 0b0011_1100, 0b1000_0000]);
    }

    #[test]
    fn micro_version_1_half_byte_padding() {
        let mut bits = Bits::new(Version::Micro(MicroVersion::M1));
        assert_eq!(bits.push_numeric_data(b"999"), Ok(()));
        assert_eq!(bits.push_terminator(EcLevel::L), Ok(()));
        assert_eq!(bits.into_bytes(), [0b0111_1111, 0b0011_1000, 0b0000_0000]);
    }

    #[test]
    fn micro_version_1_full_byte_padding() {
        let mut bits = Bits::new(Version::Micro(MicroVersion::M1));
        assert_eq!(bits.push_numeric_data(b""), Ok(()));
        assert_eq!(bits.push_terminator(EcLevel::L), Ok(()));
        assert_eq!(bits.into_bytes(), [0b0000_0000, 0b1110_1100, 0]);
    }
}
