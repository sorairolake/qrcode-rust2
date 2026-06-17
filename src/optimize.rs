// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2018 Ignas Anikevicius
// SPDX-FileCopyrightText: 2019 Atul Bhosale
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
// SPDX-FileCopyrightText: 2026 Lars Gerchow
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Find the optimal data mode sequence to encode a piece of data.

mod internal;
mod parser;
mod segment;

use alloc::{vec, vec::Vec};

pub use self::{parser::Parser, segment::Segment};
use crate::types::{Mode, Version};

/// QR code data optimizer.
#[derive(Debug)]
pub struct Optimizer<I> {
    parser: I,
    last_segment: Segment,
    last_segment_size: usize,
    version: Version,
    ended: bool,
}

impl<I: Iterator<Item = Segment>> Optimizer<I> {
    /// Optimizes the segments by combining adjacent segments when possible.
    ///
    /// Currently this method uses a greedy algorithm by combining segments from
    /// left to right until the new segment is longer than before. This method
    /// does _not_ use Annex J from the ISO standard.
    pub fn new(mut segments: I, version: Version) -> Self {
        match segments.next() {
            None => Self {
                parser: segments,
                last_segment: Segment {
                    mode: Mode::Numeric,
                    begin: 0,
                    end: 0,
                },
                last_segment_size: 0,
                version,
                ended: true,
            },
            Some(segment) => Self {
                parser: segments,
                last_segment: segment,
                last_segment_size: segment.encoded_len(version),
                version,
                ended: false,
            },
        }
    }
}

impl Parser<'_> {
    /// Creates a new `Optimizer` based on this parser.
    #[must_use]
    pub fn optimize(self, version: Version) -> Optimizer<Self> {
        Optimizer::new(self, version)
    }
}

impl<I: Iterator<Item = Segment>> Iterator for Optimizer<I> {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }

        loop {
            match self.parser.next() {
                None => {
                    self.ended = true;
                    return Some(self.last_segment);
                }
                Some(segment) => {
                    let seg_size = segment.encoded_len(self.version);

                    let new_segment = Segment {
                        mode: self.last_segment.mode.max(segment.mode),
                        begin: self.last_segment.begin,
                        end: segment.end,
                    };
                    let new_size = new_segment.encoded_len(self.version);

                    if self.last_segment_size + seg_size >= new_size {
                        self.last_segment = new_segment;
                        self.last_segment_size = new_size;
                    } else {
                        let old_segment = self.last_segment;
                        self.last_segment = segment;
                        self.last_segment_size = seg_size;
                        return Some(old_segment);
                    }
                }
            }
        }
    }
}

/// Computes the total encoded length of all segments.
#[must_use]
pub fn total_encoded_len(segments: &[Segment], version: Version) -> usize {
    segments.iter().map(|seg| seg.encoded_len(version)).sum()
}

/// Returns the optimized segmentation of `data` for the given `version`,
/// guaranteed never to be larger than the trivial single-mode encoding.
///
/// The [`Optimizer`] is a greedy left-to-right merger and is _not_ globally
/// optimal (it does not implement ISO/IEC 18004 Annex J). For low-capacity
/// Micro QR symbols a locally cheaper split can keep a numeric run separate
/// whose extra per-segment mode and character-count indicators push the total
/// over the symbol capacity, even though encoding the whole payload as a single
/// alphanumeric (or byte) segment would fit. To avoid producing an encoding
/// strictly worse than the single-mode baseline, this compares the greedy
/// result against one segment spanning the whole payload in the lowest common
/// mode and returns whichever is smaller.
#[must_use]
pub fn optimize(data: &[u8], version: Version) -> Vec<Segment> {
    optimize_segments(&Parser::new(data).collect::<Vec<_>>(), version)
}

/// Optimizes already-parsed `segments` for the given `version`, clamping the
/// greedy result to the single-mode baseline (see [`optimize`]).
///
/// Callers that try several candidate versions can parse once and reuse the
/// `segments` slice across calls.
#[must_use]
pub(crate) fn optimize_segments(segments: &[Segment], version: Version) -> Vec<Segment> {
    let greedy = Optimizer::new(segments.iter().copied(), version).collect::<Vec<_>>();

    // Single-mode baseline: one segment over the whole payload in the lowest
    // common mode that can encode every character. `Mode::max` falls back to
    // `Byte`, which can encode any data, so the baseline is always valid.
    if let Some(mode) = segments.iter().map(|seg| seg.mode).reduce(Mode::max) {
        let single = Segment {
            mode,
            begin: segments[0].begin,
            end: segments[segments.len() - 1].end,
        };
        if single.encoded_len(version) < total_encoded_len(&greedy, version) {
            return vec![single];
        }
    }

    greedy
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;

    fn optimization_result(given: &[Segment], expected: &[Segment], version: Version) {
        let prev_len = total_encoded_len(given, version);
        let opt_segs = Optimizer::new(given.iter().copied(), version).collect::<Vec<_>>();
        let new_len = total_encoded_len(&opt_segs, version);
        if given != opt_segs {
            assert!(prev_len > new_len, "{prev_len} > {new_len}");
        }
        assert!(
            opt_segs == expected,
            "Optimization gave something better: {new_len} < {} ({opt_segs:?})",
            total_encoded_len(expected, version)
        );
    }

    #[test]
    fn example_1() {
        optimization_result(
            &[
                Segment {
                    mode: Mode::Alphanumeric,
                    begin: 0,
                    end: 3,
                },
                Segment {
                    mode: Mode::Numeric,
                    begin: 3,
                    end: 6,
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 6,
                    end: 10,
                },
            ],
            &[
                Segment {
                    mode: Mode::Alphanumeric,
                    begin: 0,
                    end: 6,
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 6,
                    end: 10,
                },
            ],
            Version::Normal(1),
        );
    }

    #[test]
    fn example_2() {
        optimization_result(
            &[
                Segment {
                    mode: Mode::Numeric,
                    begin: 0,
                    end: 29,
                },
                Segment {
                    mode: Mode::Alphanumeric,
                    begin: 29,
                    end: 30,
                },
                Segment {
                    mode: Mode::Numeric,
                    begin: 30,
                    end: 32,
                },
                Segment {
                    mode: Mode::Alphanumeric,
                    begin: 32,
                    end: 35,
                },
                Segment {
                    mode: Mode::Numeric,
                    begin: 35,
                    end: 38,
                },
            ],
            &[
                Segment {
                    mode: Mode::Numeric,
                    begin: 0,
                    end: 29,
                },
                Segment {
                    mode: Mode::Alphanumeric,
                    begin: 29,
                    end: 38,
                },
            ],
            Version::Normal(9),
        );
    }

    #[test]
    fn example_3() {
        optimization_result(
            &[
                Segment {
                    mode: Mode::Kanji,
                    begin: 0,
                    end: 4,
                },
                Segment {
                    mode: Mode::Alphanumeric,
                    begin: 4,
                    end: 5,
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 5,
                    end: 6,
                },
                Segment {
                    mode: Mode::Kanji,
                    begin: 6,
                    end: 8,
                },
            ],
            &[Segment {
                mode: Mode::Byte,
                begin: 0,
                end: 8,
            }],
            Version::Normal(1),
        );
    }

    #[test]
    fn example_4() {
        optimization_result(
            &[
                Segment {
                    mode: Mode::Kanji,
                    begin: 0,
                    end: 10,
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 10,
                    end: 11,
                },
            ],
            &[
                Segment {
                    mode: Mode::Kanji,
                    begin: 0,
                    end: 10,
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 10,
                    end: 11,
                },
            ],
            Version::Normal(1),
        );
    }

    #[test]
    fn example_5() {
        optimization_result(
            &[
                Segment {
                    mode: Mode::Kanji,
                    begin: 0,
                    end: 10,
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 10,
                    end: 11,
                },
            ],
            &[
                Segment {
                    mode: Mode::Kanji,
                    begin: 0,
                    end: 10,
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 10,
                    end: 11,
                },
            ],
            Version::RectMicro(17, 139),
        );
    }

    #[test]
    fn annex_j_guideline_1a() {
        optimization_result(
            &[
                Segment {
                    mode: Mode::Numeric,
                    begin: 0,
                    end: 3,
                },
                Segment {
                    mode: Mode::Alphanumeric,
                    begin: 3,
                    end: 4,
                },
            ],
            &[
                Segment {
                    mode: Mode::Numeric,
                    begin: 0,
                    end: 3,
                },
                Segment {
                    mode: Mode::Alphanumeric,
                    begin: 3,
                    end: 4,
                },
            ],
            Version::Micro(2),
        );
    }

    #[test]
    fn annex_j_guideline_1b() {
        optimization_result(
            &[
                Segment {
                    mode: Mode::Numeric,
                    begin: 0,
                    end: 2,
                },
                Segment {
                    mode: Mode::Alphanumeric,
                    begin: 2,
                    end: 4,
                },
            ],
            &[Segment {
                mode: Mode::Alphanumeric,
                begin: 0,
                end: 4,
            }],
            Version::Micro(2),
        );
    }

    #[test]
    fn annex_j_guideline_1c() {
        optimization_result(
            &[
                Segment {
                    mode: Mode::Numeric,
                    begin: 0,
                    end: 3,
                },
                Segment {
                    mode: Mode::Alphanumeric,
                    begin: 3,
                    end: 4,
                },
            ],
            &[Segment {
                mode: Mode::Alphanumeric,
                begin: 0,
                end: 4,
            }],
            Version::Micro(3),
        );
    }

    // Regression: the greedy optimizer can split a mixed numeric/alphanumeric
    // payload into segments whose combined header overhead exceeds the symbol
    // capacity, even though a single alphanumeric segment fits. `optimize` must
    // clamp to that single-mode baseline. "9BA3935DM3TBE4" is 14 QR-alphanumeric
    // characters: greedy yields 89 bits (> M3-L's 84), one segment yields 83.
    #[test]
    fn single_mode_baseline_micro_qr() {
        let data = b"9BA3935DM3TBE4";
        let version = Version::Micro(3);

        // Greedy alone overruns the M3-L 84-bit capacity.
        let greedy = Optimizer::new(
            Parser::new(data).collect::<Vec<_>>().iter().copied(),
            version,
        )
        .collect::<Vec<_>>();
        assert!(total_encoded_len(&greedy, version) > 84);

        // The clamped optimizer collapses to a single alphanumeric segment.
        let opt = optimize(data, version);
        assert_eq!(
            opt,
            [Segment {
                mode: Mode::Alphanumeric,
                begin: 0,
                end: 14,
            }]
        );
        assert!(total_encoded_len(&opt, version) <= 84);
    }
}
