// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`EcLevel`].

/// The error correction level. It allows the original information be recovered
/// even if parts of the code is damaged.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum EcLevel {
    /// Low error correction. Allows up to 7% of wrong blocks.
    L,

    /// Medium error correction. Allows up to 15% of wrong blocks.
    #[default]
    M,

    /// Quartile error correction. Allows up to 25% of wrong blocks.
    Q,

    /// High error correction. Allows up to 30% of wrong blocks.
    H,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ec_level() {
        assert_eq!(EcLevel::L as u8, 0);
        assert_eq!(EcLevel::M as u8, 1);
        assert_eq!(EcLevel::Q as u8, 2);
        assert_eq!(EcLevel::H as u8, 3);
    }

    #[test]
    fn default_works() {
        assert_eq!(EcLevel::default(), EcLevel::M);
    }
}
