// SPDX-FileCopyrightText: 2017 kennytm
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`Color`].

use core::ops::Not;

/// The color of a module.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    /// The module is light colored.
    Light,

    /// The module is dark colored.
    Dark,
}

impl Color {
    /// Selects a value according to color of the module.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::Color;
    /// #
    /// assert_eq!(Color::Light.select(1, 0), 0);
    /// assert_eq!(Color::Dark.select("black", "white"), "black");
    /// ```
    pub fn select<T>(self, dark: T, light: T) -> T {
        match self {
            Self::Light => light,
            Self::Dark => dark,
        }
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}
