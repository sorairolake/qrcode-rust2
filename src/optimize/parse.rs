// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`Parser`].

use core::slice::Iter;

use super::{
    Segment,
    internal::{Action, ExclCharSet, STATE_TRANSITION, State},
};
use crate::types::Mode;

/// An iterator over the items of an [`ExclCharSet`].
#[derive(Debug)]
struct EcsIter<I> {
    base: I,
    index: usize,
    ended: bool,
}

impl<'a, I: Iterator<Item = &'a u8>> Iterator for EcsIter<I> {
    type Item = (usize, ExclCharSet);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }

        match self.base.next() {
            None => {
                self.ended = true;
                Some((self.index, ExclCharSet::End))
            }
            Some(c) => {
                let old_index = self.index;
                self.index += 1;
                Some((old_index, ExclCharSet::from_u8(*c)))
            }
        }
    }
}

/// QR code data parser to classify the input into distinct segments.
#[derive(Debug)]
pub struct Parser<'a> {
    ecs_iter: EcsIter<Iter<'a, u8>>,
    state: State,
    begin: usize,
    pending_single_byte: bool,
}

impl Parser<'_> {
    /// Creates a new iterator which parse the data into segments that only
    /// contains their exclusive subsets. No optimization is done at this point.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode2::{
    /// #     optimize::{Parser, Segment},
    /// #     types::Mode,
    /// # };
    /// #
    /// let parse_res = Parser::new(b"ABC123abcd").collect::<Vec<Segment>>();
    /// assert_eq!(
    ///     parse_res,
    ///     &[
    ///         Segment {
    ///             mode: Mode::Alphanumeric,
    ///             begin: 0,
    ///             end: 3
    ///         },
    ///         Segment {
    ///             mode: Mode::Numeric,
    ///             begin: 3,
    ///             end: 6
    ///         },
    ///         Segment {
    ///             mode: Mode::Byte,
    ///             begin: 6,
    ///             end: 10
    ///         }
    ///     ]
    /// );
    /// ```
    #[must_use]
    pub fn new(data: &[u8]) -> Parser<'_> {
        Parser {
            ecs_iter: EcsIter {
                base: data.iter(),
                index: 0,
                ended: false,
            },
            state: State::Init,
            begin: 0,
            pending_single_byte: false,
        }
    }
}

impl Iterator for Parser<'_> {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pending_single_byte {
            self.pending_single_byte = false;
            self.begin += 1;
            return Some(Segment {
                mode: Mode::Byte,
                begin: self.begin - 1,
                end: self.begin,
            });
        }

        loop {
            let (i, ecs) = self.ecs_iter.next()?;
            let (next_state, action) = STATE_TRANSITION[self.state as usize + ecs as usize];
            self.state = next_state;

            let old_begin = self.begin;
            let push_mode = match action {
                Action::Idle => continue,
                Action::Numeric => Mode::Numeric,
                Action::Alpha => Mode::Alphanumeric,
                Action::Byte => Mode::Byte,
                Action::Kanji => Mode::Kanji,
                Action::KanjiAndSingleByte => {
                    let next_begin = i - 1;
                    if self.begin == next_begin {
                        Mode::Byte
                    } else {
                        self.pending_single_byte = true;
                        self.begin = next_begin;
                        return Some(Segment {
                            mode: Mode::Kanji,
                            begin: old_begin,
                            end: next_begin,
                        });
                    }
                }
            };

            self.begin = i;
            return Some(Segment {
                mode: push_mode,
                begin: old_begin,
                end: i,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;

    fn parse(data: &[u8]) -> Vec<Segment> {
        Parser::new(data).collect()
    }

    #[test]
    fn test_parse_1() {
        let segs = parse(b"01049123451234591597033130128%10ABC123");
        assert_eq!(
            segs,
            &[
                Segment {
                    mode: Mode::Numeric,
                    begin: 0,
                    end: 29
                },
                Segment {
                    mode: Mode::Alphanumeric,
                    begin: 29,
                    end: 30
                },
                Segment {
                    mode: Mode::Numeric,
                    begin: 30,
                    end: 32
                },
                Segment {
                    mode: Mode::Alphanumeric,
                    begin: 32,
                    end: 35
                },
                Segment {
                    mode: Mode::Numeric,
                    begin: 35,
                    end: 38
                },
            ]
        );
    }

    #[test]
    fn test_parse_shift_jis_example_1() {
        // "あ、AｱÅ"
        let segs = parse(b"\x82\xA0\x81\x41\x41\xB1\x81\xF0");
        assert_eq!(
            segs,
            &[
                Segment {
                    mode: Mode::Kanji,
                    begin: 0,
                    end: 4
                },
                Segment {
                    mode: Mode::Alphanumeric,
                    begin: 4,
                    end: 5
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 5,
                    end: 6
                },
                Segment {
                    mode: Mode::Kanji,
                    begin: 6,
                    end: 8
                },
            ]
        );
    }

    #[test]
    fn test_parse_utf_8() {
        // Mojibake?
        let segs = parse(b"\xE3\x81\x82\xE3\x80\x81A\xEF\xBD\xB1\xE2\x84\xAB");
        assert_eq!(
            segs,
            &[
                Segment {
                    mode: Mode::Kanji,
                    begin: 0,
                    end: 4
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 4,
                    end: 5
                },
                Segment {
                    mode: Mode::Kanji,
                    begin: 5,
                    end: 7
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 7,
                    end: 10
                },
                Segment {
                    mode: Mode::Kanji,
                    begin: 10,
                    end: 12
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 12,
                    end: 13
                },
            ]
        );
    }

    #[test]
    fn test_not_kanji_1() {
        let segs = parse(b"\x81\x30");
        assert_eq!(
            segs,
            &[
                Segment {
                    mode: Mode::Byte,
                    begin: 0,
                    end: 1
                },
                Segment {
                    mode: Mode::Numeric,
                    begin: 1,
                    end: 2
                },
            ]
        );
    }

    #[test]
    fn test_not_kanji_2() {
        // Note that it's implementation detail that the byte seq is split into two.
        // Perhaps adjust the test to check for this.
        let segs = parse(b"\xEB\xC0");
        assert_eq!(
            segs,
            &[
                Segment {
                    mode: Mode::Byte,
                    begin: 0,
                    end: 1
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 1,
                    end: 2
                },
            ]
        );
    }

    #[test]
    fn test_not_kanji_3() {
        let segs = parse(b"\x81\x7F");
        assert_eq!(
            segs,
            &[
                Segment {
                    mode: Mode::Byte,
                    begin: 0,
                    end: 1
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 1,
                    end: 2
                },
            ]
        );
    }

    #[test]
    fn test_not_kanji_4() {
        let segs = parse(b"\x81\x40\x81");
        assert_eq!(
            segs,
            &[
                Segment {
                    mode: Mode::Kanji,
                    begin: 0,
                    end: 2
                },
                Segment {
                    mode: Mode::Byte,
                    begin: 2,
                    end: 3
                },
            ]
        );
    }
}
