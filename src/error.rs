// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2018 Ethan Pailes
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Error types for this crate.

use core::{error, fmt, result};

/// `Error` encodes the error encountered when generating a QR code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The data is too long to encode into a QR code for the given version.
    DataTooLong,

    /// The provided version / error correction level combination is invalid.
    InvalidVersion,

    /// Some characters in the data cannot be supported by the provided QR code
    /// version.
    UnsupportedCharacterSet,

    /// The provided ECI designator is invalid. A valid designator should be
    /// between 0 and 999,999.
    InvalidEciDesignator,

    /// A character not belonging to the character set is found.
    InvalidCharacter,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataTooLong => write!(f, "data too long"),
            Self::InvalidVersion => write!(f, "invalid version"),
            Self::UnsupportedCharacterSet => write!(f, "unsupported character set"),
            Self::InvalidEciDesignator => write!(f, "invalid ECI designator"),
            Self::InvalidCharacter => write!(f, "invalid character"),
        }
    }
}

impl error::Error for Error {}

/// `Result` is a convenient alias for a QR code generation result.
pub type Result<T> = result::Result<T, Error>;
