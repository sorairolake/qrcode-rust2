// SPDX-FileCopyrightText: 2026 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`MicroVersion`].

use crate::error::Error;

/// `MicroVersion` is a type that represents a Micro QR code version.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MicroVersion {
    /// An 11×11 Micro QR code symbol.
    M1 = 1,

    /// A 13×13 Micro QR code symbol.
    M2,

    /// A 15×15 Micro QR code symbol.
    M3,

    /// A 17×17 Micro QR code symbol.
    M4,
}

impl MicroVersion {
    /// All versions of Micro QR code.
    pub(crate) const ALL: [Self; 4] = [Self::M1, Self::M2, Self::M3, Self::M4];
}

impl From<MicroVersion> for u8 {
    fn from(version: MicroVersion) -> Self {
        version as Self
    }
}

impl TryFrom<u8> for MicroVersion {
    type Error = Error;

    fn try_from(version: u8) -> Result<Self, Self::Error> {
        match version {
            1 => Ok(Self::M1),
            2 => Ok(Self::M2),
            3 => Ok(Self::M3),
            4 => Ok(Self::M4),
            _ => Err(Error::InvalidVersion),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_micro_version_to_u8() {
        assert_eq!(u8::from(MicroVersion::M1), 1);
        assert_eq!(u8::from(MicroVersion::M2), 2);
        assert_eq!(u8::from(MicroVersion::M3), 3);
        assert_eq!(u8::from(MicroVersion::M4), 4);
    }

    #[test]
    fn try_from_u8_to_micro_version() {
        assert_eq!(MicroVersion::try_from(1).unwrap(), MicroVersion::M1);
        assert_eq!(MicroVersion::try_from(2).unwrap(), MicroVersion::M2);
        assert_eq!(MicroVersion::try_from(3).unwrap(), MicroVersion::M3);
        assert_eq!(MicroVersion::try_from(4).unwrap(), MicroVersion::M4);
    }

    #[test]
    fn try_from_u8_to_micro_version_with_invalid_version() {
        assert_eq!(
            MicroVersion::try_from(0).unwrap_err(),
            Error::InvalidVersion
        );
        assert_eq!(
            MicroVersion::try_from(5).unwrap_err(),
            Error::InvalidVersion
        );
        assert_eq!(
            MicroVersion::try_from(u8::MAX).unwrap_err(),
            Error::InvalidVersion
        );
    }
}
