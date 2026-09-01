// SPDX-FileCopyrightText: 2026 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`RectMicroVersion`].

/// `RectMicroVersion` is a type that represents a rMQR code version.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RectMicroVersion {
    /// A 7×43 rMQR code symbol.
    R7x43,

    /// A 7×59 rMQR code symbol.
    R7x59,

    /// A 7×77 rMQR code symbol.
    R7x77,

    /// A 7×99 rMQR code symbol.
    R7x99,

    /// A 7×139 rMQR code symbol.
    R7x139,

    /// A 9×43 rMQR code symbol.
    R9x43,

    /// A 9×59 rMQR code symbol.
    R9x59,

    /// A 9×77 rMQR code symbol.
    R9x77,

    /// A 9×99 rMQR code symbol.
    R9x99,

    /// A 9×139 rMQR code symbol.
    R9x139,

    /// An 11×27 rMQR code symbol.
    R11x27,

    /// An 11×43 rMQR code symbol.
    R11x43,

    /// An 11×59 rMQR code symbol.
    R11x59,

    /// An 11×77 rMQR code symbol.
    R11x77,

    /// An 11×99 rMQR code symbol.
    R11x99,

    /// An 11×139 rMQR code symbol.
    R11x139,

    /// A 13×27 rMQR code symbol.
    R13x27,

    /// A 13×43 rMQR code symbol.
    R13x43,

    /// A 13×59 rMQR code symbol.
    R13x59,

    /// A 13×77 rMQR code symbol.
    R13x77,

    /// A 13×99 rMQR code symbol.
    R13x99,

    /// A 13×139 rMQR code symbol.
    R13x139,

    /// A 15×43 rMQR code symbol.
    R15x43,

    /// A 15×59 rMQR code symbol.
    R15x59,

    /// A 15×77 rMQR code symbol.
    R15x77,

    /// A 15×99 rMQR code symbol.
    R15x99,

    /// A 15×139 rMQR code symbol.
    R15x139,

    /// A 17×43 rMQR code symbol.
    R17x43,

    /// A 17×59 rMQR code symbol.
    R17x59,

    /// A 17×77 rMQR code symbol.
    R17x77,

    /// A 17×99 rMQR code symbol.
    R17x99,

    /// A 17×139 rMQR code symbol.
    R17x139,
}
