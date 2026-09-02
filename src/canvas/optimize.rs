// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2019 Atul Bhosale
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Select mask with the lowest penalty score.

use alloc::vec::Vec;

use super::{Canvas, MaskPattern};
use crate::types::{Color, Version};

static ALL_PATTERNS_QR: [MaskPattern; 8] = [
    MaskPattern::Checkerboard,
    MaskPattern::HorizontalLines,
    MaskPattern::VerticalLines,
    MaskPattern::DiagonalLines,
    MaskPattern::LargeCheckerboard,
    MaskPattern::Fields,
    MaskPattern::Diamonds,
    MaskPattern::Meadow,
];

static ALL_PATTERNS_MICRO_QR: [MaskPattern; 4] = [
    MaskPattern::HorizontalLines,
    MaskPattern::LargeCheckerboard,
    MaskPattern::Diamonds,
    MaskPattern::Meadow,
];

static ALL_PATTERNS_RMQR: [MaskPattern; 1] = [MaskPattern::LargeCheckerboard];

impl Canvas {
    #[expect(clippy::missing_panics_doc)]
    /// Constructs a new canvas and apply the best masking that gives the lowest
    /// penalty score.
    #[must_use]
    pub fn apply_best_mask(&self) -> Self {
        match self.version {
            Version::Normal(_) => ALL_PATTERNS_QR.iter(),
            Version::Micro(_) => ALL_PATTERNS_MICRO_QR.iter(),
            Version::RectMicro(_) => ALL_PATTERNS_RMQR.iter(),
        }
        .map(|ptn| {
            let mut c = self.clone();
            c.apply_mask(*ptn);
            c
        })
        .min_by_key(Self::compute_total_penalty_scores)
        .unwrap()
    }

    /// Converts the modules into a vector of colors.
    pub fn into_colors(self) -> Vec<Color> {
        self.modules.into_iter().map(Color::from).collect()
    }
}
