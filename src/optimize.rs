// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2018 Ignas Anikevicius
// SPDX-FileCopyrightText: 2019 Atul Bhosale
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Find the optimal data mode sequence to encode a piece of data.

mod internal;
mod parser;
mod segment;

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
}
