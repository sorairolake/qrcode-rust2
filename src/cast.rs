// SPDX-FileCopyrightText: 2017 kennytm
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Utilities for casting primitive integer types.

pub trait Truncate {
    fn truncate_as_u8(self) -> u8;
}

impl Truncate for u16 {
    #[expect(clippy::cast_possible_truncation)]
    // TODO: use `u16::truncate()` when stable.
    // <https://github.com/rust-lang/rust/issues/154330>
    fn truncate_as_u8(self) -> u8 {
        self as u8
    }
}
