// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The `types` module contains types associated with the functional elements of
//! a QR code.

mod color;
mod ec_level;
mod mode;
mod version;

pub use self::{color::Color, ec_level::EcLevel, mode::Mode, version::Version};
