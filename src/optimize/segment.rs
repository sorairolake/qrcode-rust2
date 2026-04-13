// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Segment.

use crate::types::{Mode, Version};

/// A segment of data committed to an encoding mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Segment {
    /// The encoding mode of the segment of data.
    pub mode: Mode,

    /// The start index of the segment.
    pub begin: usize,

    /// The end index (exclusive) of the segment.
    pub end: usize,
}

impl Segment {
    /// Computes the number of bits (including the size of the mode indicator
    /// and length bits) when this segment is encoded.
    #[must_use]
    pub fn encoded_len(&self, version: Version) -> usize {
        let byte_size = self.end - self.begin;
        let chars_count = if self.mode == Mode::Kanji {
            byte_size / 2
        } else {
            byte_size
        };

        let mode_bits_count = version.mode_bits_count();
        let length_bits_count = self.mode.length_bits_count(version);
        let data_bits_count = self.mode.data_bits_count(chars_count);

        mode_bits_count + length_bits_count + data_bits_count
    }
}
