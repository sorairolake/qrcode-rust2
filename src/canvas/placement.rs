// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2018 Ignas Anikevicius
// SPDX-FileCopyrightText: 2023 Nakanishi
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of features related to the data placement.

use super::{Canvas, Module, data_module_iter::DataModuleIter};
use crate::types::{Color, EcLevel, Version};

impl Canvas {
    fn draw_codewords<I>(&mut self, codewords: &[u8], is_half_codeword_at_end: bool, coords: &mut I)
    where
        I: Iterator<Item = (i16, i16)>,
    {
        let length = codewords.len();
        let last_word = if is_half_codeword_at_end {
            length - 1
        } else {
            length
        };
        for (i, b) in codewords.iter().enumerate() {
            let bits_end = if i == last_word { 4 } else { 0 };
            'outside: for j in (bits_end..=7).rev() {
                let color = if (*b & (1 << j)) == 0 {
                    Color::Light
                } else {
                    Color::Dark
                };
                for (x, y) in coords.by_ref() {
                    let r = self.get_mut(x, y);
                    if *r == Module::Empty {
                        *r = Module::Unmasked(color);
                        continue 'outside;
                    }
                }
                return;
            }
        }
    }

    /// Draws the encoded data and error correction codes to the empty modules.
    pub fn draw_data(&mut self, data: &[u8], ec: &[u8]) {
        let is_half_codeword_at_end = matches!(
            (self.version, self.ec_level),
            (Version::Micro(1 | 3), EcLevel::L) | (Version::Micro(3), EcLevel::M)
        );
        let mut coords = DataModuleIter::new(self.version);
        self.draw_codewords(data, is_half_codeword_at_end, &mut coords);
        self.draw_codewords(ec, false, &mut coords);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_micro_qr_1() {
        let mut c = Canvas::new(Version::Micro(1), EcLevel::L);
        c.draw_all_functional_patterns();
        c.draw_data(b"\x6E\x5D\xE2", b"\x2B\x63");
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######.#.#\n",
                "#.....#..-*\n",
                "#.###.#..**\n",
                "#.###.#..*-\n",
                "#.###.#..**\n",
                "#.....#..*-\n",
                "#######..*-\n",
                ".........-*\n",
                "#........**\n",
                ".***-**---*\n",
                "#---*-*-**-"
            )
        );
    }

    #[test]
    fn test_qr_2() {
        let mut c = Canvas::new(Version::Normal(2), EcLevel::L);
        c.draw_all_functional_patterns();
        c.draw_data(
            &[
                0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49,
                0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92,
                0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24, 0x92, 0x49, 0x24,
                0x92, 0x49, 0x24,
            ],
            b"",
        );
        assert_eq!(
            c.to_debug_str(),
            concat!(
                "\n",
                "#######..--*---*-.#######\n",
                "#.....#..-*-*-*-*.#.....#\n",
                "#.###.#..*---*---.#.###.#\n",
                "#.###.#..--*---*-.#.###.#\n",
                "#.###.#..-*-*-*-*.#.###.#\n",
                "#.....#..*---*---.#.....#\n",
                "#######.#.#.#.#.#.#######\n",
                ".........--*---*-........\n",
                "......#..-*-*-*-*........\n",
                "--*-*-.-**---*---*--**--*\n",
                "-*-*--#----*---*---------\n",
                "*----*.*--*-*-*-*-**--**-\n",
                "--*-*-#-**---*---*--**--*\n",
                "-*-*--.----*---*---------\n",
                "*----*#*--*-*-*-*-**--**-\n",
                "--*-*-.-**---*---*--**--*\n",
                "-*-*--#----*---*#####----\n",
                "........#-*-*-*-#...#-**-\n",
                "#######..*---*--#.#.#*--*\n",
                "#.....#..--*---*#...#----\n",
                "#.###.#..-*-*-*-#####-**-\n",
                "#.###.#..*---*--*----*--*\n",
                "#.###.#..--*------**-----\n",
                "#.....#..-*-*-**-*--*-**-\n",
                "#######..*---*--*----*--*"
            )
        );
    }

    #[test]
    fn test_rmqr() {
        let mut c = Canvas::new(Version::RectMicro(7, 77), EcLevel::M);
        c.draw_all_functional_patterns();
        c.draw_data(
            &[
                0x71, 0x68, 0x74, 0x74, 0x70, 0x73, 0x3A, 0x2F, 0x2F, 0x6F, 0x75, 0x64, 0x6F, 0x6E,
                0x2E, 0x78, 0x79, 0x7A, 0x00, 0xEC, 0xFF, 0x6B, 0xC6, 0xCB, 0x02, 0x06, 0xA5, 0xFE,
                0x36, 0x6E, 0x55, 0xFF,
            ],
            b"",
        );
        assert_eq!(
            c.to_debug_str_mask_same(),
            concat!(
                "\n",
                "#######.#.#.#.#.#.#.#.#.###.#.#.#.#.#.#.#.#.#.#.#.###.#.#.#.#.#.#.#.#.#.#.###\n",
                "#.....#.#..#######.##.#.#.#..##.####.#.####.#######.#.##..#..###..........#.#\n",
                "#.###.#..#####.#..#...#####.##..##....##.#.##..##.####.##.##....###.#..######\n",
                "#.###.#.###.##.###...#..#.....######..#...##.##.#.#.#####.#....#.#####..#...#\n",
                "#.###.#.##.??#.##.####..###..####..#..#.#..###..#.#####.##.###.#.#.##.###.#.#\n",
                "#.....#.###???..######..#.#.##.#.###...###...##..##.#.#..##..###.....##.#...#\n",
                "#######.#.#.#.#.#.#.#.#.###.#.#.#.#.#.#.#.#.#.#.#.###.#.#.#.#.#.#.#.#.#.#####"
            )
        );
    }
}
