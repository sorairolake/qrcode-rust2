// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of features related to the corner finder patterns for rMQR
//! code.

use super::Canvas;
use crate::types::Color;

impl Canvas {
    /// Draws the rMQR corner finder pattern.
    pub(super) fn draw_corner_finder_pattern(&mut self) {
        if !self.version.is_rect_micro() {
            return;
        }
        // Bottom left
        self.put(0, -1, Color::Dark);
        self.put(1, -1, Color::Dark);
        self.put(2, -1, Color::Dark);

        // Top right
        self.put(-1, 0, Color::Dark);
        self.put(-1, 1, Color::Dark);
        self.put(-2, 0, Color::Dark);
        self.put(-2, 1, Color::Light);

        if self.height >= 11 {
            self.put(0, -2, Color::Dark);
            self.put(1, -2, Color::Light);
        }
    }
}
