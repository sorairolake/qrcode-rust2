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

/// Returns the optimal segmentation of `data` for the given `version`.
///
/// Unlike the greedy [`Optimizer`], this computes a globally optimal mode
/// segmentation (ISO/IEC 18004 Annex J) and never produces an encoding larger
/// than necessary — including never larger than the trivial single-mode one.
#[must_use]
pub fn optimize(data: &[u8], version: Version) -> Vec<Segment> {
    optimize_segments(&Parser::new(data).collect::<Vec<_>>(), version)
}

/// Computes the optimal segmentation of already-parsed `segments` for the given
/// `version` (see [`optimize`]).
///
/// This is a dynamic program over run boundaries: `dp[b]` is the minimum number
/// of encoded bits for the first `b` parsed runs, and each transition considers
/// merging a contiguous span of runs `[a, b)` into a single segment whose mode
/// is the lowest common mode able to encode every character in the span. The
/// per-segment cost is the exact bit count from [`Segment::encoded_len`].
///
/// Optimal segment boundaries are always a subset of run boundaries — splitting
/// a maximal same-class run only adds segment-header overhead with no density
/// benefit — so restricting the search to run boundaries loses no optimality.
/// The greedy result and the single-mode encoding are both candidate
/// segmentations this dominates.
///
/// Runs in `O(k^2)` time and `O(k)` space, where `k` is the number of parsed
/// runs (typically small). Callers that try several candidate versions can
/// parse once and reuse the `segments` slice across calls.
#[must_use]
pub(crate) fn optimize_segments(segments: &[Segment], version: Version) -> Vec<Segment> {
    let runs = segments.len();
    if runs == 0 {
        return Vec::new();
    }

    // `dp[b]` = minimum total encoded bits for the first `b` runs; `back[b]` =
    // (start run, segment mode) of the last segment on the cheapest path to
    // `b`.
    let mut dp = vec![usize::MAX; runs + 1];
    let mut back = vec![(0_usize, Mode::Numeric); runs + 1];
    dp[0] = 0;

    for b in 1..=runs {
        // Extend a single segment leftwards over runs `[a, b)`, tracking the
        // lowest common mode that can encode every run in the span. Seed from
        // an actual run mode (not `Mode::Numeric`): `Mode::max` falls
        // back to `Byte` for incomparable modes, so seeding with
        // `Numeric` would misclassify a pure-Kanji span as `Byte`.
        let mut mode = segments[b - 1].mode;
        for a in (0..b).rev() {
            mode = mode.max(segments[a].mode);
            if dp[a] == usize::MAX {
                continue;
            }
            let segment = Segment {
                mode,
                begin: segments[a].begin,
                end: segments[b - 1].end,
            };
            let cost = dp[a] + segment.encoded_len(version);
            if cost < dp[b] {
                dp[b] = cost;
                back[b] = (a, mode);
            }
        }
    }

    // Reconstruct the segments from the backpointers.
    let mut result = Vec::new();
    let mut b = runs;
    while b > 0 {
        let (a, mode) = back[b];
        result.push(Segment {
            mode,
            begin: segments[a].begin,
            end: segments[b - 1].end,
        });
        b = a;
    }
    result.reverse();
    result
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
    // capacity, even though a single alphanumeric segment fits. The optimal
    // segmentation must keep it as one segment. "9BA3935DM3TBE4" is 14
    // QR-alphanumeric characters: greedy yields 89 bits (> M3-L's 84), one
    // segment yields 83.
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

        // The optimal segmentation collapses to a single alphanumeric segment.
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

    /// Independent reference: the minimum encoded length over *every* way to
    /// partition the parsed runs into contiguous single-mode segments, found by
    /// exhaustive enumeration (each segment uses the lowest common mode of its
    /// runs). Exponential in the run count, so only for small test inputs.
    fn brute_force_min(segments: &[Segment], version: Version) -> usize {
        let runs = segments.len();
        if runs == 0 {
            return 0;
        }
        let mut best = usize::MAX;
        // Bit `i` set => a segment boundary after run `i` (run `runs - 1`
        // always ends a segment).
        for mask in 0..(1u32 << (runs - 1)) {
            let mut total = 0;
            let mut start = 0;
            for i in 0..runs {
                let boundary = i == runs - 1 || (mask >> i) & 1 == 1;
                if boundary {
                    let mut mode = segments[start].mode;
                    for run in &segments[start..=i] {
                        mode = mode.max(run.mode);
                    }
                    total += Segment {
                        mode,
                        begin: segments[start].begin,
                        end: segments[i].end,
                    }
                    .encoded_len(version);
                    start = i + 1;
                }
            }
            best = best.min(total);
        }
        best
    }

    // The DP must equal the exhaustive optimum, and (since greedy and the
    // single-mode encoding are both candidate segmentations) never exceed
    // either of them.
    #[test]
    fn dp_is_optimal() {
        const INPUTS: &[&[u8]] = &[
            b"9BA3935DM3TBE4",
            b"A1B2C3D4E5F6G7",
            b"HELLO123WORLD456",
            b"1234ABCD5678EF",
            b"AB000000000000000000CD",
            b"a1B2c3d4",
            b"foo BAR 123 baz 456",
        ];
        for &data in INPUTS {
            for version in [Version::Normal(1), Version::Micro(3), Version::Micro(4)] {
                let parsed = Parser::new(data).collect::<Vec<_>>();
                let opt = optimize(data, version);
                let opt_len = total_encoded_len(&opt, version);

                assert_eq!(
                    opt_len,
                    brute_force_min(&parsed, version),
                    "{:?} {version:?}: DP != exhaustive optimum",
                    core::str::from_utf8(data).unwrap()
                );

                let greedy = Optimizer::new(parsed.iter().copied(), version).collect::<Vec<_>>();
                assert!(opt_len <= total_encoded_len(&greedy, version));

                let single_mode = parsed.iter().map(|s| s.mode).reduce(Mode::max).unwrap();
                let single = Segment {
                    mode: single_mode,
                    begin: parsed[0].begin,
                    end: parsed[parsed.len() - 1].end,
                };
                assert!(opt_len <= single.encoded_len(version));
            }
        }
    }

    // The DP strictly improves on greedy (scattered short digit runs) and on
    // the single-mode encoding (a long numeric run worth isolating), each
    // in a case where the other is already optimal — confirming it is not
    // just one of them.
    #[test]
    fn dp_beats_greedy_and_single_mode() {
        // Greedy over-splits and overruns; the optimum is one alphanumeric
        // segment.
        let data = b"9BA3935DM3TBE4";
        let version = Version::Micro(3);
        let parsed = Parser::new(data).collect::<Vec<_>>();
        let greedy = Optimizer::new(parsed.iter().copied(), version).collect::<Vec<_>>();
        assert!(
            total_encoded_len(&optimize(data, version), version)
                < total_encoded_len(&greedy, version)
        );

        // A long numeric run: the single alphanumeric segment is far from
        // optimal.
        let data = b"AB000000000000000000CD";
        let version = Version::Normal(1);
        let parsed = Parser::new(data).collect::<Vec<_>>();
        let single_mode = parsed.iter().map(|s| s.mode).reduce(Mode::max).unwrap();
        let single = Segment {
            mode: single_mode,
            begin: 0,
            end: data.len(),
        };
        assert!(total_encoded_len(&optimize(data, version), version) < single.encoded_len(version));
    }
}
