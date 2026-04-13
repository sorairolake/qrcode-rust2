// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Internal types and data for parsing.

/// All values of [`u8`] can be split into 9 different character sets when
/// determining which encoding to use. This enum represents these groupings for
/// parsing purpose.
#[derive(Clone, Copy)]
pub enum ExclCharSet {
    /// The end of string.
    End = 0,

    /// All symbols supported by the Alphanumeric encoding, i.e. space, `$`,
    /// `%`, `*`, `+`, `-`, `.`, `/` and `:`.
    Symbol = 1,

    /// All numbers (0–9).
    Numeric = 2,

    /// All uppercase letters (A–Z). These characters may also appear in the
    /// second byte of a Shift JIS 2-byte encoding.
    Alpha = 3,

    /// The first byte of a Shift JIS 2-byte encoding, in the range 0x81–0x9F.
    KanjiHi1 = 4,

    /// The first byte of a Shift JIS 2-byte encoding, in the range 0xE0–0xEA.
    KanjiHi2 = 5,

    /// The first byte of a Shift JIS 2-byte encoding, of value 0xEB. This is
    /// different from the other two range that the second byte has a smaller
    /// range.
    KanjiHi3 = 6,

    /// The second byte of a Shift JIS 2-byte encoding, in the range 0x40–0xBF,
    /// excluding letters (covered by `Alpha`), 0x81–0x9F (covered by
    /// `KanjiHi1`), and the invalid byte 0x7F.
    KanjiLo1 = 7,

    /// The second byte of a Shift JIS 2-byte encoding, in the range 0xC0–0xFC,
    /// excluding the range 0xE0–0xEB (covered by `KanjiHi2` and `KanjiHi3`).
    /// This half of byte-pair cannot appear as the second byte leaded by
    /// `KanjiHi3`.
    KanjiLo2 = 8,

    /// Any other values not covered by the above character sets.
    Byte = 9,
}

impl ExclCharSet {
    /// Determines which character set a byte is in.
    pub const fn from_u8(c: u8) -> Self {
        match c {
            0x20 | 0x24 | 0x25 | 0x2A | 0x2B | 0x2D..=0x2F | 0x3A => Self::Symbol,
            0x30..=0x39 => Self::Numeric,
            0x41..=0x5A => Self::Alpha,
            0x81..=0x9F => Self::KanjiHi1,
            0xE0..=0xEA => Self::KanjiHi2,
            0xEB => Self::KanjiHi3,
            0x40 | 0x5B..=0x7E | 0x80 | 0xA0..=0xBF => Self::KanjiLo1,
            0xC0..=0xDF | 0xEC..=0xFC => Self::KanjiLo2,
            _ => Self::Byte,
        }
    }
}

/// The current parsing state.
#[derive(Clone, Copy, Debug)]
pub enum State {
    /// Just initialized.
    Init = 0,

    /// Inside a string that can be exclusively encoded as Numeric.
    Numeric = 10,

    /// Inside a string that can be exclusively encoded as Alphanumeric.
    Alpha = 20,

    /// Inside a string that can be exclusively encoded as 8-Bit Byte.
    Byte = 30,

    /// Just encountered the first byte of a Shift JIS 2-byte sequence of the
    /// set `KanjiHi1` or `KanjiHi2`.
    KanjiHi12 = 40,

    /// Just encountered the first byte of a Shift JIS 2-byte sequence of the
    /// set `KanjiHi3`.
    KanjiHi3 = 50,

    /// Inside a string that can be exclusively encoded as Kanji.
    Kanji = 60,
}

/// What should the parser do after a state transition.
#[derive(Clone, Copy)]
pub enum Action {
    /// The parser should do nothing.
    Idle,

    /// Push the current segment as a Numeric string, and reset the marks.
    Numeric,

    /// Push the current segment as an Alphanumeric string, and reset the marks.
    Alpha,

    /// Push the current segment as a 8-Bit Byte string, and reset the marks.
    Byte,

    /// Push the current segment as a Kanji string, and reset the marks.
    Kanji,

    /// Push the current segment excluding the last byte as a Kanji string, then
    /// push the remaining single byte as a Byte string, and reset the marks.
    KanjiAndSingleByte,
}

// STATE_TRANSITION[current_state + next_character] == (next_state, what_to_do)
pub static STATE_TRANSITION: [(State, Action); 70] = [
    // Init state:
    // End
    (State::Init, Action::Idle),
    // Symbol
    (State::Alpha, Action::Idle),
    // Numeric
    (State::Numeric, Action::Idle),
    // Alpha
    (State::Alpha, Action::Idle),
    // KanjiHi1
    (State::KanjiHi12, Action::Idle),
    // KanjiHi2
    (State::KanjiHi12, Action::Idle),
    // KanjiHi3
    (State::KanjiHi3, Action::Idle),
    // KanjiLo1
    (State::Byte, Action::Idle),
    // KanjiLo2
    (State::Byte, Action::Idle),
    // Byte
    (State::Byte, Action::Idle),
    // Numeric state:
    // End
    (State::Init, Action::Numeric),
    // Symbol
    (State::Alpha, Action::Numeric),
    // Numeric
    (State::Numeric, Action::Idle),
    // Alpha
    (State::Alpha, Action::Numeric),
    // KanjiHi1
    (State::KanjiHi12, Action::Numeric),
    // KanjiHi2
    (State::KanjiHi12, Action::Numeric),
    // KanjiHi3
    (State::KanjiHi3, Action::Numeric),
    // KanjiLo1
    (State::Byte, Action::Numeric),
    // KanjiLo2
    (State::Byte, Action::Numeric),
    // Byte
    (State::Byte, Action::Numeric),
    // Alpha state:
    // End
    (State::Init, Action::Alpha),
    // Symbol
    (State::Alpha, Action::Idle),
    // Numeric
    (State::Numeric, Action::Alpha),
    // Alpha
    (State::Alpha, Action::Idle),
    // KanjiHi1
    (State::KanjiHi12, Action::Alpha),
    // KanjiHi2
    (State::KanjiHi12, Action::Alpha),
    // KanjiHi3
    (State::KanjiHi3, Action::Alpha),
    // KanjiLo1
    (State::Byte, Action::Alpha),
    // KanjiLo2
    (State::Byte, Action::Alpha),
    // Byte
    (State::Byte, Action::Alpha),
    // Byte state:
    // End
    (State::Init, Action::Byte),
    // Symbol
    (State::Alpha, Action::Byte),
    // Numeric
    (State::Numeric, Action::Byte),
    // Alpha
    (State::Alpha, Action::Byte),
    // KanjiHi1
    (State::KanjiHi12, Action::Byte),
    // KanjiHi2
    (State::KanjiHi12, Action::Byte),
    // KanjiHi3
    (State::KanjiHi3, Action::Byte),
    // KanjiLo1
    (State::Byte, Action::Idle),
    // KanjiLo2
    (State::Byte, Action::Idle),
    // Byte
    (State::Byte, Action::Idle),
    // KanjiHi12 state:
    // End
    (State::Init, Action::KanjiAndSingleByte),
    // Symbol
    (State::Alpha, Action::KanjiAndSingleByte),
    // Numeric
    (State::Numeric, Action::KanjiAndSingleByte),
    // Alpha
    (State::Kanji, Action::Idle),
    // KanjiHi1
    (State::Kanji, Action::Idle),
    // KanjiHi2
    (State::Kanji, Action::Idle),
    // KanjiHi3
    (State::Kanji, Action::Idle),
    // KanjiLo1
    (State::Kanji, Action::Idle),
    // KanjiLo2
    (State::Kanji, Action::Idle),
    // Byte
    (State::Byte, Action::KanjiAndSingleByte),
    // KanjiHi3 state:
    // End
    (State::Init, Action::KanjiAndSingleByte),
    // Symbol
    (State::Alpha, Action::KanjiAndSingleByte),
    // Numeric
    (State::Numeric, Action::KanjiAndSingleByte),
    // Alpha
    (State::Kanji, Action::Idle),
    // KanjiHi1
    (State::Kanji, Action::Idle),
    // KanjiHi2
    (State::KanjiHi12, Action::KanjiAndSingleByte),
    // KanjiHi3
    (State::KanjiHi3, Action::KanjiAndSingleByte),
    // KanjiLo1
    (State::Kanji, Action::Idle),
    // KanjiLo2
    (State::Byte, Action::KanjiAndSingleByte),
    // Byte
    (State::Byte, Action::KanjiAndSingleByte),
    // Kanji state:
    // End
    (State::Init, Action::Kanji),
    // Symbol
    (State::Alpha, Action::Kanji),
    // Numeric
    (State::Numeric, Action::Kanji),
    // Alpha
    (State::Alpha, Action::Kanji),
    // KanjiHi1
    (State::KanjiHi12, Action::Idle),
    // KanjiHi2
    (State::KanjiHi12, Action::Idle),
    // KanjiHi3
    (State::KanjiHi3, Action::Idle),
    // KanjiLo1
    (State::Byte, Action::Kanji),
    // KanjiLo2
    (State::Byte, Action::Kanji),
    // Byte
    (State::Byte, Action::Kanji),
];
