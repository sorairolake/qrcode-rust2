// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Number of allowed errors.

use super::error_correction_sizes::{DATA_BYTES_PER_BLOCK, EC_BYTES_PER_BLOCK};
use crate::types::{EcLevel, QrResult, Version};

/// Computes the maximum allowed number of erratic modules can be introduced to
/// the QR code, before the data becomes truly corrupted.
///
/// # Errors
///
/// Returns [`Err`] if it is not valid to use the `ec_level` for the given
/// version (e.g. [`Version::Micro(1)`](Version::Micro) with [`EcLevel::H`]).
///
/// # Examples
///
/// ```
/// # use qrcode2::{EcLevel, Version, ec};
/// #
/// assert_eq!(
///     ec::max_allowed_errors(Version::Normal(40), EcLevel::M),
///     Ok(686)
/// );
/// assert_eq!(ec::max_allowed_errors(Version::Micro(4), EcLevel::Q), Ok(7));
/// assert_eq!(
///     ec::max_allowed_errors(Version::RectMicro(17, 139), EcLevel::H),
///     Ok(78)
/// );
/// ```
pub fn max_allowed_errors(version: Version, ec_level: EcLevel) -> QrResult<usize> {
    let p = match (version, ec_level) {
        (Version::Micro(2) | Version::Normal(1), EcLevel::L) => 3,
        (Version::Micro(_) | Version::Normal(2), EcLevel::L)
        | (Version::Micro(2) | Version::Normal(1), EcLevel::M) => 2,
        (Version::Normal(1), _) | (Version::Normal(3), EcLevel::L) => 1,
        _ => 0,
    };

    let ec_bytes_per_block = version.fetch(ec_level, &EC_BYTES_PER_BLOCK)?;
    let (_, count1, _, count2) = version.fetch(ec_level, &DATA_BYTES_PER_BLOCK)?;
    let ec_bytes = (count1 + count2) * ec_bytes_per_block;

    Ok((ec_bytes - p) / 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_versions() {
        assert_eq!(
            max_allowed_errors(Version::Normal(1), EcLevel::L).unwrap(),
            2
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(1), EcLevel::M).unwrap(),
            4
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(1), EcLevel::Q).unwrap(),
            6
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(1), EcLevel::H).unwrap(),
            8
        );

        assert_eq!(
            max_allowed_errors(Version::Normal(2), EcLevel::L).unwrap(),
            4
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(2), EcLevel::M).unwrap(),
            8
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(2), EcLevel::Q).unwrap(),
            11
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(2), EcLevel::H).unwrap(),
            14
        );

        assert_eq!(
            max_allowed_errors(Version::Normal(3), EcLevel::L).unwrap(),
            7
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(3), EcLevel::M).unwrap(),
            13
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(3), EcLevel::Q).unwrap(),
            18
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(3), EcLevel::H).unwrap(),
            22
        );

        assert_eq!(
            max_allowed_errors(Version::Normal(4), EcLevel::L).unwrap(),
            10
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(4), EcLevel::M).unwrap(),
            18
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(4), EcLevel::Q).unwrap(),
            26
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(4), EcLevel::H).unwrap(),
            32
        );

        assert_eq!(
            max_allowed_errors(Version::Micro(1), EcLevel::L).unwrap(),
            0
        );

        assert_eq!(
            max_allowed_errors(Version::Micro(2), EcLevel::L).unwrap(),
            1
        );
        assert_eq!(
            max_allowed_errors(Version::Micro(2), EcLevel::M).unwrap(),
            2
        );

        assert_eq!(
            max_allowed_errors(Version::Micro(3), EcLevel::L).unwrap(),
            2
        );
        assert_eq!(
            max_allowed_errors(Version::Micro(3), EcLevel::M).unwrap(),
            4
        );

        assert_eq!(
            max_allowed_errors(Version::Micro(4), EcLevel::L).unwrap(),
            3
        );
        assert_eq!(
            max_allowed_errors(Version::Micro(4), EcLevel::M).unwrap(),
            5
        );
        assert_eq!(
            max_allowed_errors(Version::Micro(4), EcLevel::Q).unwrap(),
            7
        );

        assert_eq!(
            max_allowed_errors(Version::RectMicro(7, 43), EcLevel::M).unwrap(),
            3
        );
        assert_eq!(
            max_allowed_errors(Version::RectMicro(7, 43), EcLevel::H).unwrap(),
            5
        );

        assert_eq!(
            max_allowed_errors(Version::RectMicro(7, 59), EcLevel::M).unwrap(),
            4
        );
        assert_eq!(
            max_allowed_errors(Version::RectMicro(7, 59), EcLevel::H).unwrap(),
            7
        );

        assert_eq!(
            max_allowed_errors(Version::RectMicro(7, 77), EcLevel::M).unwrap(),
            6
        );
        assert_eq!(
            max_allowed_errors(Version::RectMicro(7, 77), EcLevel::H).unwrap(),
            11
        );

        assert_eq!(
            max_allowed_errors(Version::RectMicro(7, 99), EcLevel::M).unwrap(),
            8
        );
        assert_eq!(
            max_allowed_errors(Version::RectMicro(7, 99), EcLevel::H).unwrap(),
            15
        );
    }

    #[test]
    fn test_high_versions() {
        assert_eq!(
            max_allowed_errors(Version::Normal(40), EcLevel::L).unwrap(),
            375
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(40), EcLevel::M).unwrap(),
            686
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(40), EcLevel::Q).unwrap(),
            1020
        );
        assert_eq!(
            max_allowed_errors(Version::Normal(40), EcLevel::H).unwrap(),
            1215
        );

        assert_eq!(
            max_allowed_errors(Version::RectMicro(17, 139), EcLevel::M).unwrap(),
            40
        );
        assert_eq!(
            max_allowed_errors(Version::RectMicro(17, 139), EcLevel::H).unwrap(),
            78
        );
    }
}
