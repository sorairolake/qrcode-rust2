// SPDX-FileCopyrightText: 2026 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`MicroVersion`].

/// `MicroVersion` is a type that represents a Micro QR code version.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MicroVersion {
    /// An 11×11 Micro QR code symbol.
    M1,

    /// A 13×13 Micro QR code symbol.
    M2,

    /// A 15×15 Micro QR code symbol.
    M3,

    /// A 17×17 Micro QR code symbol.
    M4,
}
