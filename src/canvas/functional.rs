// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2019 Atul Bhosale
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! All functional patterns before data placement.

use super::{Canvas, alignment::ALIGNMENT_PATTERN_POSITIONS};
use crate::types::Version;

impl Canvas {
    /// Draws all functional patterns, before data placement.
    ///
    /// All functional patterns (e.g. the finder pattern) _except_ the format
    /// info pattern will be filled in. The format info pattern will be filled
    /// with light modules instead. Data bits can then put in the empty modules.
    /// with [`Canvas::draw_data`].
    pub fn draw_all_functional_patterns(&mut self) {
        self.draw_finder_patterns();
        self.draw_alignment_patterns();
        self.draw_reserved_format_info_patterns();
        self.draw_timing_patterns();
        self.draw_corner_finder_pattern();
        self.draw_alignment_patterns_rmqr();
        self.draw_version_info_patterns();
    }
}

/// Gets whether the module at the given coordinates represents a functional
/// module.
#[must_use]
pub fn is_functional(version: Version, width: i16, x: i16, y: i16) -> bool {
    debug_assert_eq!(width, version.width());

    let x = if x < 0 { x + width } else { x };
    let y = if y < 0 { y + width } else { y };

    match version {
        Version::Micro(_) => x == 0 || y == 0 || (x < 9 && y < 9),
        Version::RectMicro(_) => unimplemented!(),
        Version::Normal(a) => {
            let a = u8::from(a);
            let timing_patterns = x == 6 || y == 6;
            let top_left_finder_pattern = x < 9 && y < 9;
            let bottom_left_finder_pattern = x < 9 && y >= width - 8;
            let top_right_finder_pattern = x >= width - 8 && y < 9;
            let non_alignment_test = timing_patterns
                || top_left_finder_pattern
                || bottom_left_finder_pattern
                || top_right_finder_pattern;
            match a {
                _ if non_alignment_test => true,
                1 => false,
                2..=6 => (width - 7 - x).abs() <= 2 && (width - 7 - y).abs() <= 2,
                _ => {
                    let positions = ALIGNMENT_PATTERN_POSITIONS[usize::from(a - 7)];
                    let last = positions.len() - 1;
                    for (i, align_x) in positions.iter().enumerate() {
                        for (j, align_y) in positions.iter().enumerate() {
                            if i == 0 && (j == 0 || j == last) || (i == last && j == 0) {
                                continue;
                            }
                            if (*align_x - x).abs() <= 2 && (*align_y - y).abs() <= 2 {
                                return true;
                            }
                        }
                    }
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EcLevel, MicroVersion, NormalVersion};

    #[test]
    fn all_functional_patterns_qr() {
        let mut c = Canvas::new(Version::Normal(NormalVersion::V2), EcLevel::L);
        c.draw_all_functional_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######..????????.#######\n",
                "#.....#..????????.#.....#\n",
                "#.###.#..????????.#.###.#\n",
                "#.###.#..????????.#.###.#\n",
                "#.###.#..????????.#.###.#\n",
                "#.....#..????????.#.....#\n",
                "#######.#.#.#.#.#.#######\n",
                ".........????????........\n",
                "......#..????????........\n",
                "??????.??????????????????\n",
                "??????#??????????????????\n",
                "??????.??????????????????\n",
                "??????#??????????????????\n",
                "??????.??????????????????\n",
                "??????#??????????????????\n",
                "??????.??????????????????\n",
                "??????#?????????#####????\n",
                "........#???????#...#????\n",
                "#######..???????#.#.#????\n",
                "#.....#..???????#...#????\n",
                "#.###.#..???????#####????\n",
                "#.###.#..????????????????\n",
                "#.###.#..????????????????\n",
                "#.....#..????????????????\n",
                "#######..????????????????"
            )
        );
    }

    #[test]
    fn all_functional_patterns_micro_qr() {
        let mut c = Canvas::new(Version::Micro(MicroVersion::M1), EcLevel::L);
        c.draw_all_functional_patterns();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######.#.#\n",
                "#.....#..??\n",
                "#.###.#..??\n",
                "#.###.#..??\n",
                "#.###.#..??\n",
                "#.....#..??\n",
                "#######..??\n",
                ".........??\n",
                "#........??\n",
                ".??????????\n",
                "#??????????"
            )
        );
    }

    #[test]
    fn is_functional_qr_1() {
        let version = Version::Normal(NormalVersion::V1);
        assert!(is_functional(version, version.width(), 0, 0));
        assert!(is_functional(version, version.width(), 10, 6));
        assert!(!is_functional(version, version.width(), 10, 5));
        assert!(!is_functional(version, version.width(), 14, 14));
        assert!(is_functional(version, version.width(), 6, 11));
        assert!(!is_functional(version, version.width(), 4, 11));
        assert!(is_functional(version, version.width(), 4, 13));
        assert!(is_functional(version, version.width(), 17, 7));
        assert!(!is_functional(version, version.width(), 17, 17));
    }

    #[test]
    fn is_functional_qr_3() {
        let version = Version::Normal(NormalVersion::V3);
        assert!(is_functional(version, version.width(), 0, 0));
        assert!(!is_functional(version, version.width(), 25, 24));
        assert!(is_functional(version, version.width(), 24, 24));
        assert!(!is_functional(version, version.width(), 9, 25));
        assert!(!is_functional(version, version.width(), 20, 0));
        assert!(is_functional(version, version.width(), 21, 0));
    }

    #[test]
    fn is_functional_qr_7() {
        let version = Version::Normal(NormalVersion::V7);
        assert!(is_functional(version, version.width(), 21, 4));
        assert!(is_functional(version, version.width(), 7, 21));
        assert!(is_functional(version, version.width(), 22, 22));
        assert!(is_functional(version, version.width(), 8, 8));
        assert!(!is_functional(version, version.width(), 19, 5));
        assert!(!is_functional(version, version.width(), 36, 3));
        assert!(!is_functional(version, version.width(), 4, 36));
        assert!(is_functional(version, version.width(), 38, 38));
    }

    #[test]
    fn is_functional_micro() {
        let version = Version::Micro(MicroVersion::M1);
        assert!(is_functional(version, version.width(), 8, 0));
        assert!(is_functional(version, version.width(), 10, 0));
        assert!(!is_functional(version, version.width(), 10, 1));
        assert!(is_functional(version, version.width(), 8, 8));
        assert!(is_functional(version, version.width(), 0, 9));
        assert!(!is_functional(version, version.width(), 1, 9));
    }
}
