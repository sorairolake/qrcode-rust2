// SPDX-FileCopyrightText: 2026 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`NormalVersion`].

/// `NormalVersion` is a type that represents a normal QR code version.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NormalVersion {
    /// A 21×21 normal QR code symbol.
    V1,

    /// A 25×25 normal QR code symbol.
    V2,

    /// A 29×29 normal QR code symbol.
    V3,

    /// A 33×33 normal QR code symbol.
    V4,

    /// A 37×37 normal QR code symbol.
    V5,

    /// A 41×41 normal QR code symbol.
    V6,

    /// A 45×45 normal QR code symbol.
    V7,

    /// A 49×49 normal QR code symbol.
    V8,

    /// A 53×53 normal QR code symbol.
    V9,

    /// A 57×57 normal QR code symbol.
    V10,

    /// A 61×61 normal QR code symbol.
    V11,

    /// A 65×65 normal QR code symbol.
    V12,

    /// A 69×69 normal QR code symbol.
    V13,

    /// A 73×73 normal QR code symbol.
    V14,

    /// A 77×77 normal QR code symbol.
    V15,

    /// An 81×81 normal QR code symbol.
    V16,

    /// An 85×85 normal QR code symbol.
    V17,

    /// A 89×89 normal QR code symbol.
    V18,

    /// An 93×93 normal QR code symbol.
    V19,

    /// A 97×97 normal QR code symbol.
    V20,

    /// A 101×101 normal QR code symbol.
    V21,

    /// A 105×105 normal QR code symbol.
    V22,

    /// A 109×109 normal QR code symbol.
    V23,

    /// A 113×113 normal QR code symbol.
    V24,

    /// A 117×117 normal QR code symbol.
    V25,

    /// A 121×121 normal QR code symbol.
    V26,

    /// A 125×125 normal QR code symbol.
    V27,

    /// A 129×129 normal QR code symbol.
    V28,

    /// A 133×133 normal QR code symbol.
    V29,

    /// A 137×137 normal QR code symbol.
    V30,

    /// A 141×141 normal QR code symbol.
    V31,

    /// A 145×145 normal QR code symbol.
    V32,

    /// A 149×149 normal QR code symbol.
    V33,

    /// A 153×153 normal QR code symbol.
    V34,

    /// A 157×157 normal QR code symbol.
    V35,

    /// A 161×161 normal QR code symbol.
    V36,

    /// A 165×165 normal QR code symbol.
    V37,

    /// A 169×169 normal QR code symbol.
    V38,

    /// A 173×173 normal QR code symbol.
    V39,

    /// A 177×177 normal QR code symbol.
    V40,
}
