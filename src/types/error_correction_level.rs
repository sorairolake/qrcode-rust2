// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Error correction level.

/// The error correction level. It allows the original information be recovered
/// even if parts of the code is damaged.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum EcLevel {
    /// Low error correction. Allows up to 7% of wrong blocks.
    L = 0,

    /// Medium error correction. Allows up to 15% of wrong blocks.
    #[default]
    M = 1,

    /// "Quartile" error correction. Allows up to 25% of wrong blocks.
    Q = 2,

    /// High error correction. Allows up to 30% of wrong blocks.
    H = 3,
}

#[cfg(test)]
mod ec_level_tests {
    use super::*;

    #[test]
    fn test_default() {
        assert_eq!(EcLevel::default(), EcLevel::M);
    }
}
