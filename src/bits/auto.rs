// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2019 Ivan Tham
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Encode the data into [`Bits`] with auto version minimization.

use alloc::vec::Vec;

use super::{Bits, terminator::DATA_LENGTHS};
use crate::{
    cast::As,
    error::{Error, Result},
    optimize::{self, Optimizer, Parser, Segment},
    types::{EcLevel, Version},
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
/// # use qrcode2::{EcLevel, Version, bits};
/// #
/// let bits = bits::encode_auto(b"Hello, world!", EcLevel::M).unwrap();
/// assert_eq!(bits.version(), Version::Normal(1));
/// ```
pub fn encode_auto(data: &[u8], ec_level: EcLevel) -> Result<Bits> {
    let segments = Parser::new(data).collect::<Vec<Segment>>();
    for version in &[Version::Normal(9), Version::Normal(26), Version::Normal(40)] {
        let opt_segments = Optimizer::new(segments.iter().copied(), *version).collect::<Vec<_>>();
        let total_len = optimize::total_encoded_len(&opt_segments, *version);
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
pub fn encode_auto_micro(data: &[u8], ec_level: EcLevel) -> Result<Bits> {
    let segments = Parser::new(data).collect::<Vec<Segment>>();
    let mut possible_versions = Vec::new();
    for version in 1..=4 {
        let version = Version::Micro(version);
        let opt_segments = Optimizer::new(segments.iter().copied(), version).collect::<Vec<_>>();
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
        let mut bits = Bits::new(*version);
        let opt_segments = Optimizer::new(segments.iter().copied(), *version).collect::<Vec<_>>();
        bits.reserve(optimize::total_encoded_len(&opt_segments, *version));
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
) -> Result<Bits> {
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
    Err(Error::DataTooLong)
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
