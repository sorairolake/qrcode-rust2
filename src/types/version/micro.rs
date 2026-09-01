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
