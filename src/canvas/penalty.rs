// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2018 Ignas Anikevicius
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of features related to the penalty score.

use alloc::boxed::Box;
use core::{cmp, iter};

use super::{Canvas, Module};
use crate::{
    cast::As,
    types::{Color, Version},
};

impl Canvas {
    /// Computes the penalty score for having too many adjacent modules with the
    /// same color.
    ///
    /// Every 5+N adjacent modules in the same column/row having the same color
    /// will contribute 3+N points.
    fn compute_adjacent_penalty_score(&self, is_horizontal: bool) -> u16 {
        let mut total_score = 0;

        for i in 0..self.width {
            let map_fn = |j| {
                if is_horizontal {
                    self.get(j, i)
                } else {
                    self.get(i, j)
                }
            };

            let colors = (0..self.width).map(map_fn).chain(iter::once(Module::Empty));
            let mut last_color = Module::Empty;
            let mut consecutive_len = 1_u16;

            for color in colors {
                if color == last_color {
                    consecutive_len += 1;
                } else {
                    last_color = color;
                    if consecutive_len >= 5 {
                        total_score += consecutive_len - 2;
                    }
                    consecutive_len = 1;
                }
            }
        }

        total_score
    }

    /// Computes the penalty score for having too many rectangles with the same
    /// color.
    ///
    /// Every 2×2 blocks (with overlapping counted) having the same color will
    /// contribute 3 points.
    fn compute_block_penalty_score(&self) -> u16 {
        let mut total_score = 0;

        for i in 0..self.width - 1 {
            for j in 0..self.width - 1 {
                let this = self.get(i, j);
                let right = self.get(i + 1, j);
                let bottom = self.get(i, j + 1);
                let bottom_right = self.get(i + 1, j + 1);
                if this == right && right == bottom && bottom == bottom_right {
                    total_score += 3;
                }
            }
        }

        total_score
    }

    /// Computes the penalty score for having a pattern similar to the finder
    /// pattern in the wrong place.
    ///
    /// Every pattern that looks like `#.###.#....` in any orientation will add
    /// 40 points.
    fn compute_finder_penalty_score(&self, is_horizontal: bool) -> u16 {
        static PATTERN: [Color; 7] = [
            Color::Dark,
            Color::Light,
            Color::Dark,
            Color::Dark,
            Color::Dark,
            Color::Light,
            Color::Dark,
        ];

        let mut total_score = 0;

        for i in 0..self.width {
            for j in 0..self.width - 6 {
                // TODO a ref to a closure should be enough?
                let get: Box<dyn Fn(i16) -> Color> = if is_horizontal {
                    Box::new(|k| self.get(k, i).into())
                } else {
                    Box::new(|k| self.get(i, k).into())
                };

                if (j..(j + 7)).map(&get).ne(PATTERN.iter().copied()) {
                    continue;
                }

                let check = |k| 0 <= k && k < self.width && get(k) != Color::Light;
                if !((j - 4)..j).any(&check) || !((j + 7)..(j + 11)).any(&check) {
                    total_score += 40;
                }
            }
        }

        total_score - 360
    }

    /// Computes the penalty score for having an unbalanced dark/light ratio.
    ///
    /// The score is given linearly by the deviation from a 50% ratio of dark
    /// modules. The highest possible score is 100.
    ///
    /// <div class="warning">
    ///
    /// Note that this algorithm differs slightly from the standard we do not
    /// round the result every 5%, but the difference should be negligible and
    /// should not affect which mask is chosen.
    ///
    /// </div>
    fn compute_balance_penalty_score(&self) -> u16 {
        let dark_modules = self.modules.iter().filter(|m| m.is_dark()).count();
        let total_modules = self.modules.len();
        let ratio = dark_modules * 200 / total_modules;
        ratio.abs_diff(100).as_u16()
    }

    /// Computes the penalty score for having too many light modules on the
    /// sides.
    ///
    /// This penalty score is exclusive to Micro QR code.
    ///
    /// <div class="warning">
    ///
    /// Note that the standard gives the formula for *efficiency* score, which
    /// has the inverse meaning of this method, but it is very easy to convert
    /// between the two (this score is (16×width − standard-score)).
    ///
    /// </div>
    fn compute_light_side_penalty_score(&self) -> u16 {
        let h = (1..self.width)
            .filter(|j| !self.get(*j, -1).is_dark())
            .count();
        let v = (1..self.width)
            .filter(|j| !self.get(-1, *j).is_dark())
            .count();

        (h + v + 15 * cmp::max(h, v)).as_u16()
    }

    /// Computes the total penalty scores. A QR code having higher points is
    /// less desirable.
    pub(super) fn compute_total_penalty_scores(&self) -> u16 {
        match self.version {
            Version::Normal(_) => {
                let s1_a = self.compute_adjacent_penalty_score(true);
                let s1_b = self.compute_adjacent_penalty_score(false);
                let s2 = self.compute_block_penalty_score();
                let s3_a = self.compute_finder_penalty_score(true);
                let s3_b = self.compute_finder_penalty_score(false);
                let s4 = self.compute_balance_penalty_score();
                s1_a + s1_b + s2 + s3_a + s3_b + s4
            }
            Version::Micro(_) => self.compute_light_side_penalty_score(),
            Version::RectMicro(..) => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::MaskPattern, *};
    use crate::types::EcLevel;

    fn create_test_canvas() -> Canvas {
        let mut c = Canvas::new(Version::Normal(1), EcLevel::Q);
        c.draw_all_functional_patterns();
        c.draw_data(
            b"\x20\x5B\x0B\x78\xD1\x72\xDC\x4D\x43\x40\xEC\x11\x00",
            b"\xA8\x48\x16\x52\xD9\x36\x9C\x00\x2E\x0F\xB4\x7A\x10",
        );
        c.apply_mask(MaskPattern::Checkerboard);
        c
    }

    #[test]
    fn check_penalty_canvas() {
        let c = create_test_canvas();
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######.##....#######\n",
                "#.....#.#..#..#.....#\n",
                "#.###.#.#..##.#.###.#\n",
                "#.###.#.#.....#.###.#\n",
                "#.###.#.#.#...#.###.#\n",
                "#.....#...#...#.....#\n",
                "#######.#.#.#.#######\n",
                "........#............\n",
                ".##.#.##....#.#.#####\n",
                ".#......####....#...#\n",
                "..##.###.##...#.##...\n",
                ".##.##.#..##.#.#.###.\n",
                "#...#.#.#.###.###.#.#\n",
                "........##.#..#...#.#\n",
                "#######.#.#....#.##..\n",
                "#.....#..#.##.##.#...\n",
                "#.###.#.#.#...#######\n",
                "#.###.#..#.#.#.#...#.\n",
                "#.###.#.#...####.#..#\n",
                "#.....#.#.##.#...#.##\n",
                "#######.....####....#"
            )
        );
    }

    #[test]
    fn test_penalty_score_adjacent() {
        let c = create_test_canvas();
        assert_eq!(c.compute_adjacent_penalty_score(true), 88);
        assert_eq!(c.compute_adjacent_penalty_score(false), 92);
    }

    #[test]
    fn test_penalty_score_block() {
        let c = create_test_canvas();
        assert_eq!(c.compute_block_penalty_score(), 90);
    }

    #[test]
    fn test_penalty_score_finder() {
        let c = create_test_canvas();
        assert_eq!(c.compute_finder_penalty_score(true), 0);
        assert_eq!(c.compute_finder_penalty_score(false), 40);
    }

    #[test]
    fn test_penalty_score_balance() {
        let c = create_test_canvas();
        assert_eq!(c.compute_balance_penalty_score(), 2);
    }

    #[test]
    fn test_penalty_score_light_sides() {
        static HORIZONTAL_SIDE: [Color; 17] = [
            Color::Dark,
            Color::Light,
            Color::Light,
            Color::Dark,
            Color::Dark,
            Color::Dark,
            Color::Light,
            Color::Light,
            Color::Dark,
            Color::Light,
            Color::Dark,
            Color::Light,
            Color::Light,
            Color::Dark,
            Color::Light,
            Color::Light,
            Color::Light,
        ];
        static VERTICAL_SIDE: [Color; 17] = [
            Color::Dark,
            Color::Dark,
            Color::Dark,
            Color::Light,
            Color::Light,
            Color::Dark,
            Color::Dark,
            Color::Light,
            Color::Dark,
            Color::Light,
            Color::Dark,
            Color::Light,
            Color::Dark,
            Color::Light,
            Color::Light,
            Color::Dark,
            Color::Light,
        ];

        let mut c = Canvas::new(Version::Micro(4), EcLevel::Q);
        for i in 0_i16..17 {
            c.put(i, -1, HORIZONTAL_SIDE[i.as_usize()]);
            c.put(-1, i, VERTICAL_SIDE[i.as_usize()]);
        }

        assert_eq!(c.compute_light_side_penalty_score(), 168);
    }
}
