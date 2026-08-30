// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`Module`].

use crate::types::Color;

/// The color of a module (pixel) in the QR code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Module {
    /// The module is empty.
    Empty,

    /// The module is of functional patterns which cannot be masked, or pixels
    /// which have been masked.
    Masked(Color),

    /// The module is of data and error correction bits before masking.
    Unmasked(Color),
}

impl From<Module> for Color {
    fn from(module: Module) -> Self {
        match module {
            Module::Empty => Self::Light,
            Module::Masked(c) | Module::Unmasked(c) => c,
        }
    }
}

impl Module {
    /// Checks whether a module is dark.
    #[must_use]
    pub fn is_dark(self) -> bool {
        Color::from(self) == Color::Dark
    }

    /// Applies a mask to the unmasked modules.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{Color, canvas::Module};
    ///
    /// assert_eq!(
    ///     Module::Unmasked(Color::Light).mask(true),
    ///     Module::Masked(Color::Dark)
    /// );
    /// assert_eq!(
    ///     Module::Unmasked(Color::Dark).mask(true),
    ///     Module::Masked(Color::Light)
    /// );
    /// assert_eq!(
    ///     Module::Unmasked(Color::Light).mask(false),
    ///     Module::Masked(Color::Light)
    /// );
    /// assert_eq!(
    ///     Module::Masked(Color::Dark).mask(true),
    ///     Module::Masked(Color::Dark)
    /// );
    /// assert_eq!(
    ///     Module::Masked(Color::Dark).mask(false),
    ///     Module::Masked(Color::Dark)
    /// );
    /// ```
    #[must_use]
    pub fn mask(self, should_invert: bool) -> Self {
        match (self, should_invert) {
            (Self::Empty, true) => Self::Masked(Color::Dark),
            (Self::Empty, false) => Self::Masked(Color::Light),
            (Self::Unmasked(c), true) => Self::Masked(!c),
            (Self::Unmasked(c), false) | (Self::Masked(c), _) => Self::Masked(c),
        }
    }
}
