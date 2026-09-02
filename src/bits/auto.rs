// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2019 Ivan Tham
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
// SPDX-FileCopyrightText: 2026 Lars Gerchow
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Encode the data into [`Bits`] with auto version minimization.

use alloc::vec::Vec;

use super::{Bits, terminator::DATA_LENGTHS};
use crate::{
    error::{Error, Result},
    optimize::{self, Parser, Segment},
    types::{EcLevel, MicroVersion, NormalVersion, RectMicroVersion, Version},
};

#[expect(clippy::missing_panics_doc)]
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
/// use qrcode2::{EcLevel, NormalVersion, Version, bits};
///
/// let bits = bits::encode_auto(b"Hello, world!", EcLevel::M).unwrap();
/// assert_eq!(bits.version(), Version::Normal(NormalVersion::V1));
/// ```
pub fn encode_auto(data: &[u8], ec_level: EcLevel) -> Result<Bits> {
    let segments = Parser::new(data).collect::<Vec<Segment>>();
    let versions = [NormalVersion::V9, NormalVersion::V26, NormalVersion::V40];
    for version in versions {
        let version = Version::Normal(version);
        let opt_segments = optimize::optimize_segments(&segments, version);
        let total_len = optimize::total_encoded_len(&opt_segments, version);
        let data_capacity = version.fetch(ec_level, &DATA_LENGTHS).unwrap();
        if total_len <= data_capacity {
            let min_version = find_min_version(total_len, ec_level);
            let mut bits = Bits::new(min_version);
            bits.reserve(total_len);
            bits.push_segments(data, opt_segments.into_iter())?;
            bits.push_terminator(ec_level)?;
            return Ok(bits);
        }
    }
    Err(Error::DataTooLong)
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
    let base = u8::try_from(base).unwrap();
    let version = NormalVersion::try_from(base + 1).unwrap();
    Version::Normal(version)
}

#[cfg(test)]
mod encode_auto_tests {
    use super::*;

    #[test]
    fn find_min_version_works() {
        assert_eq!(
            find_min_version(60, EcLevel::L),
            Version::Normal(NormalVersion::V1)
        );
        assert_eq!(
            find_min_version(200, EcLevel::L),
            Version::Normal(NormalVersion::V2)
        );
        assert_eq!(
            find_min_version(200, EcLevel::H),
            Version::Normal(NormalVersion::V3)
        );
        assert_eq!(
            find_min_version(20000, EcLevel::L),
            Version::Normal(NormalVersion::V37)
        );
        assert_eq!(
            find_min_version(640, EcLevel::L),
            Version::Normal(NormalVersion::V4)
        );
        assert_eq!(
            find_min_version(641, EcLevel::L),
            Version::Normal(NormalVersion::V5)
        );
        assert_eq!(
            find_min_version(999_999, EcLevel::H),
            Version::Normal(NormalVersion::V40)
        );
    }

    #[test]
    fn alpha_q() {
        let bits = encode_auto(b"HELLO WORLD", EcLevel::Q).unwrap();
        assert_eq!(bits.version(), Version::Normal(NormalVersion::V1));
    }

    #[test]
    fn alpha_h() {
        let bits = encode_auto(b"HELLO WORLD", EcLevel::H).unwrap();
        assert_eq!(bits.version(), Version::Normal(NormalVersion::V2));
    }

    #[test]
    fn mixed() {
        let bits = encode_auto(b"This is a mixed data test. 1234567890", EcLevel::H).unwrap();
        assert_eq!(bits.version(), Version::Normal(NormalVersion::V4));
    }
}

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
/// use qrcode2::{EcLevel, MicroVersion, Version, bits};
///
/// let bits = bits::encode_auto_micro(b"Hello, world!", EcLevel::M).unwrap();
/// assert_eq!(bits.version(), Version::Micro(MicroVersion::M4));
/// ```
pub fn encode_auto_micro(data: &[u8], ec_level: EcLevel) -> Result<Bits> {
    let segments = Parser::new(data).collect::<Vec<Segment>>();
    let mut possible_versions = Vec::new();
    for version in MicroVersion::ALL {
        let version = Version::Micro(version);
        let opt_segments = optimize::optimize_segments(&segments, version);
        let total_len = optimize::total_encoded_len(&opt_segments, version);
        let data_capacity = version.fetch(ec_level, &DATA_LENGTHS);
        if let Ok(capacity) = data_capacity
            && total_len <= capacity
        {
            possible_versions.push(version);
            break;
        }
    }

    let min_version = possible_versions.iter().min_by_key(|v| v.width());

    if let Some(version) = min_version {
        let mut bits = Bits::new(version);
        let opt_segments = optimize::optimize_segments(&segments, version);
        bits.reserve(optimize::total_encoded_len(&opt_segments, version));
        bits.push_segments(data, opt_segments.into_iter())?;
        bits.push_terminator(ec_level)?;
        return Ok(bits);
    }
    Err(Error::DataTooLong)
}

#[cfg(test)]
mod encode_auto_micro_tests {
    use super::*;

    #[test]
    fn alpha_l() {
        let bits = encode_auto_micro(b"HELLO WORLD", EcLevel::L).unwrap();
        assert_eq!(bits.version(), Version::Micro(MicroVersion::M3));
    }

    #[test]
    fn alpha_q() {
        let bits = encode_auto_micro(b"HELLO WORLD", EcLevel::Q).unwrap();
        assert_eq!(bits.version(), Version::Micro(MicroVersion::M4));
    }

    #[test]
    fn mixed() {
        let bits = encode_auto_micro(b"Mixed. 1234567890", EcLevel::M).unwrap();
        assert_eq!(bits.version(), Version::Micro(MicroVersion::M4));
    }
}

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
/// use qrcode2::{
///     EcLevel, RectMicroVersion, Version,
///     bits::{self, RectMicroStrategy},
/// };
///
/// let bits = bits::encode_auto_rect_micro(b"Hello, world!", EcLevel::M, RectMicroStrategy::Area)
///     .unwrap();
/// assert_eq!(bits.version(), Version::RectMicro(RectMicroVersion::R11x43));
/// ```
pub fn encode_auto_rect_micro(
    data: &[u8],
    ec_level: EcLevel,
    strategy: RectMicroStrategy,
) -> Result<Bits> {
    let segments = Parser::new(data).collect::<Vec<Segment>>();
    let mut possible_versions = Vec::new();
    let mut skip_height = None;
    for version in RectMicroVersion::ALL {
        let version = Version::RectMicro(version);
        let current_height = version.height();
        if let Some(sh) = skip_height {
            if current_height == sh {
                continue;
            }
            skip_height = None;
        }
        let opt_segments = optimize::optimize_segments(&segments, version);
        let total_len = optimize::total_encoded_len(&opt_segments, version);
        let data_capacity = version.fetch(ec_level, &DATA_LENGTHS)?;
        if total_len <= data_capacity {
            possible_versions.push(version);
            skip_height = Some(current_height);
        }
    }

    let min_version = match strategy {
        RectMicroStrategy::Width => possible_versions.iter().min_by_key(|v| v.width()),
        // `possible_versions` is already sorted by height.
        RectMicroStrategy::Height => possible_versions.first(),
        RectMicroStrategy::Area => possible_versions
            .iter()
            .min_by_key(|v| v.height() * v.width()),
    };

    if let Some(version) = min_version {
        let mut bits = Bits::new(version);
        let opt_segments = optimize::optimize_segments(&segments, version);
        bits.reserve(optimize::total_encoded_len(&opt_segments, version));
        bits.push_segments(data, opt_segments.into_iter())?;
        bits.push_terminator(ec_level)?;
        return Ok(bits);
    }
    Err(Error::DataTooLong)
}

#[cfg(test)]
mod encode_auto_rect_micro_tests {
    use super::*;

    #[test]
    fn alpha_m_width() {
        let bits =
            encode_auto_rect_micro(b"HELLO WORLD", EcLevel::M, RectMicroStrategy::Width).unwrap();
        assert_eq!(bits.version(), Version::RectMicro(RectMicroVersion::R13x27));
    }

    #[test]
    fn alpha_m_height() {
        let bits =
            encode_auto_rect_micro(b"HELLO WORLD", EcLevel::M, RectMicroStrategy::Height).unwrap();
        assert_eq!(bits.version(), Version::RectMicro(RectMicroVersion::R7x59));
    }

    #[test]
    fn alpha_m_area() {
        let bits =
            encode_auto_rect_micro(b"HELLO WORLD", EcLevel::M, RectMicroStrategy::Area).unwrap();
        assert_eq!(bits.version(), Version::RectMicro(RectMicroVersion::R13x27));
    }

    #[test]
    fn alpha_h_width() {
        let bits =
            encode_auto_rect_micro(b"HELLO WORLD", EcLevel::H, RectMicroStrategy::Width).unwrap();
        assert_eq!(bits.version(), Version::RectMicro(RectMicroVersion::R11x43));
    }

    #[test]
    fn alpha_h_height() {
        let bits =
            encode_auto_rect_micro(b"HELLO WORLD", EcLevel::H, RectMicroStrategy::Height).unwrap();
        assert_eq!(bits.version(), Version::RectMicro(RectMicroVersion::R7x77));
    }

    #[test]
    fn alpha_h_area() {
        let bits =
            encode_auto_rect_micro(b"HELLO WORLD", EcLevel::H, RectMicroStrategy::Area).unwrap();
        assert_eq!(bits.version(), Version::RectMicro(RectMicroVersion::R11x43));
    }

    #[test]
    fn mixed_width() {
        let bits = encode_auto_rect_micro(
            b"This is a mixed data test. 1234567890",
            EcLevel::H,
            RectMicroStrategy::Width,
        )
        .unwrap();
        assert_eq!(bits.version(), Version::RectMicro(RectMicroVersion::R17x77));
    }

    #[test]
    fn mixed_height() {
        let bits = encode_auto_rect_micro(
            b"This is a mixed data test. 1234567890",
            EcLevel::H,
            RectMicroStrategy::Height,
        )
        .unwrap();
        assert_eq!(
            bits.version(),
            Version::RectMicro(RectMicroVersion::R11x139)
        );
    }

    #[test]
    fn mixed_area() {
        let bits = encode_auto_rect_micro(
            b"This is a mixed data test. 1234567890",
            EcLevel::H,
            RectMicroStrategy::Area,
        )
        .unwrap();
        assert_eq!(bits.version(), Version::RectMicro(RectMicroVersion::R13x99));
    }
}
