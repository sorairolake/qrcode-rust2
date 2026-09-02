// SPDX-FileCopyrightText: 2026 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Implementation of [`NormalVersion`].

use crate::error::Error;

/// `NormalVersion` is a type that represents a normal QR code version.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NormalVersion {
    /// A 21×21 normal QR code symbol.
    V1 = 1,

    /// A 25×25 normal QR code symbol.
    V2,

    /// A 29×29 normal QR code symbol.
    V3,

    /// A 33×33 normal QR code symbol.
    V4,

    /// A 37×37 normal QR code symbol.
    V5,

    /// A 41×41 normal QR code symbol.
    V6,

    /// A 45×45 normal QR code symbol.
    V7,

    /// A 49×49 normal QR code symbol.
    V8,

    /// A 53×53 normal QR code symbol.
    V9,

    /// A 57×57 normal QR code symbol.
    V10,

    /// A 61×61 normal QR code symbol.
    V11,

    /// A 65×65 normal QR code symbol.
    V12,

    /// A 69×69 normal QR code symbol.
    V13,

    /// A 73×73 normal QR code symbol.
    V14,

    /// A 77×77 normal QR code symbol.
    V15,

    /// An 81×81 normal QR code symbol.
    V16,

    /// An 85×85 normal QR code symbol.
    V17,

    /// A 89×89 normal QR code symbol.
    V18,

    /// An 93×93 normal QR code symbol.
    V19,

    /// A 97×97 normal QR code symbol.
    V20,

    /// A 101×101 normal QR code symbol.
    V21,

    /// A 105×105 normal QR code symbol.
    V22,

    /// A 109×109 normal QR code symbol.
    V23,

    /// A 113×113 normal QR code symbol.
    V24,

    /// A 117×117 normal QR code symbol.
    V25,

    /// A 121×121 normal QR code symbol.
    V26,

    /// A 125×125 normal QR code symbol.
    V27,

    /// A 129×129 normal QR code symbol.
    V28,

    /// A 133×133 normal QR code symbol.
    V29,

    /// A 137×137 normal QR code symbol.
    V30,

    /// A 141×141 normal QR code symbol.
    V31,

    /// A 145×145 normal QR code symbol.
    V32,

    /// A 149×149 normal QR code symbol.
    V33,

    /// A 153×153 normal QR code symbol.
    V34,

    /// A 157×157 normal QR code symbol.
    V35,

    /// A 161×161 normal QR code symbol.
    V36,

    /// A 165×165 normal QR code symbol.
    V37,

    /// A 169×169 normal QR code symbol.
    V38,

    /// A 173×173 normal QR code symbol.
    V39,

    /// A 177×177 normal QR code symbol.
    V40,
}

impl From<NormalVersion> for u8 {
    fn from(version: NormalVersion) -> Self {
        version as Self
    }
}

impl TryFrom<u8> for NormalVersion {
    type Error = Error;

    fn try_from(version: u8) -> Result<Self, Self::Error> {
        match version {
            1 => Ok(Self::V1),
            2 => Ok(Self::V2),
            3 => Ok(Self::V3),
            4 => Ok(Self::V4),
            5 => Ok(Self::V5),
            6 => Ok(Self::V6),
            7 => Ok(Self::V7),
            8 => Ok(Self::V8),
            9 => Ok(Self::V9),
            10 => Ok(Self::V10),
            11 => Ok(Self::V11),
            12 => Ok(Self::V12),
            13 => Ok(Self::V13),
            14 => Ok(Self::V14),
            15 => Ok(Self::V15),
            16 => Ok(Self::V16),
            17 => Ok(Self::V17),
            18 => Ok(Self::V18),
            19 => Ok(Self::V19),
            20 => Ok(Self::V20),
            21 => Ok(Self::V21),
            22 => Ok(Self::V22),
            23 => Ok(Self::V23),
            24 => Ok(Self::V24),
            25 => Ok(Self::V25),
            26 => Ok(Self::V26),
            27 => Ok(Self::V27),
            28 => Ok(Self::V28),
            29 => Ok(Self::V29),
            30 => Ok(Self::V30),
            31 => Ok(Self::V31),
            32 => Ok(Self::V32),
            33 => Ok(Self::V33),
            34 => Ok(Self::V34),
            35 => Ok(Self::V35),
            36 => Ok(Self::V36),
            37 => Ok(Self::V37),
            38 => Ok(Self::V38),
            39 => Ok(Self::V39),
            40 => Ok(Self::V40),
            _ => Err(Error::InvalidVersion),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_normal_version_to_u8() {
        assert_eq!(u8::from(NormalVersion::V1), 1);
        assert_eq!(u8::from(NormalVersion::V2), 2);
        assert_eq!(u8::from(NormalVersion::V3), 3);
        assert_eq!(u8::from(NormalVersion::V4), 4);
        assert_eq!(u8::from(NormalVersion::V5), 5);
        assert_eq!(u8::from(NormalVersion::V6), 6);
        assert_eq!(u8::from(NormalVersion::V7), 7);
        assert_eq!(u8::from(NormalVersion::V8), 8);
        assert_eq!(u8::from(NormalVersion::V9), 9);
        assert_eq!(u8::from(NormalVersion::V10), 10);
        assert_eq!(u8::from(NormalVersion::V11), 11);
        assert_eq!(u8::from(NormalVersion::V12), 12);
        assert_eq!(u8::from(NormalVersion::V13), 13);
        assert_eq!(u8::from(NormalVersion::V14), 14);
        assert_eq!(u8::from(NormalVersion::V15), 15);
        assert_eq!(u8::from(NormalVersion::V16), 16);
        assert_eq!(u8::from(NormalVersion::V17), 17);
        assert_eq!(u8::from(NormalVersion::V18), 18);
        assert_eq!(u8::from(NormalVersion::V19), 19);
        assert_eq!(u8::from(NormalVersion::V20), 20);
        assert_eq!(u8::from(NormalVersion::V21), 21);
        assert_eq!(u8::from(NormalVersion::V22), 22);
        assert_eq!(u8::from(NormalVersion::V23), 23);
        assert_eq!(u8::from(NormalVersion::V24), 24);
        assert_eq!(u8::from(NormalVersion::V25), 25);
        assert_eq!(u8::from(NormalVersion::V26), 26);
        assert_eq!(u8::from(NormalVersion::V27), 27);
        assert_eq!(u8::from(NormalVersion::V28), 28);
        assert_eq!(u8::from(NormalVersion::V29), 29);
        assert_eq!(u8::from(NormalVersion::V30), 30);
        assert_eq!(u8::from(NormalVersion::V31), 31);
        assert_eq!(u8::from(NormalVersion::V32), 32);
        assert_eq!(u8::from(NormalVersion::V33), 33);
        assert_eq!(u8::from(NormalVersion::V34), 34);
        assert_eq!(u8::from(NormalVersion::V35), 35);
        assert_eq!(u8::from(NormalVersion::V36), 36);
        assert_eq!(u8::from(NormalVersion::V37), 37);
        assert_eq!(u8::from(NormalVersion::V38), 38);
        assert_eq!(u8::from(NormalVersion::V39), 39);
        assert_eq!(u8::from(NormalVersion::V40), 40);
    }

    #[test]
    fn try_from_u8_to_normal_version() {
        assert_eq!(NormalVersion::try_from(1).unwrap(), NormalVersion::V1);
        assert_eq!(NormalVersion::try_from(2).unwrap(), NormalVersion::V2);
        assert_eq!(NormalVersion::try_from(3).unwrap(), NormalVersion::V3);
        assert_eq!(NormalVersion::try_from(4).unwrap(), NormalVersion::V4);
        assert_eq!(NormalVersion::try_from(5).unwrap(), NormalVersion::V5);
        assert_eq!(NormalVersion::try_from(6).unwrap(), NormalVersion::V6);
        assert_eq!(NormalVersion::try_from(7).unwrap(), NormalVersion::V7);
        assert_eq!(NormalVersion::try_from(8).unwrap(), NormalVersion::V8);
        assert_eq!(NormalVersion::try_from(9).unwrap(), NormalVersion::V9);
        assert_eq!(NormalVersion::try_from(10).unwrap(), NormalVersion::V10);
        assert_eq!(NormalVersion::try_from(11).unwrap(), NormalVersion::V11);
        assert_eq!(NormalVersion::try_from(12).unwrap(), NormalVersion::V12);
        assert_eq!(NormalVersion::try_from(13).unwrap(), NormalVersion::V13);
        assert_eq!(NormalVersion::try_from(14).unwrap(), NormalVersion::V14);
        assert_eq!(NormalVersion::try_from(15).unwrap(), NormalVersion::V15);
        assert_eq!(NormalVersion::try_from(16).unwrap(), NormalVersion::V16);
        assert_eq!(NormalVersion::try_from(17).unwrap(), NormalVersion::V17);
        assert_eq!(NormalVersion::try_from(18).unwrap(), NormalVersion::V18);
        assert_eq!(NormalVersion::try_from(19).unwrap(), NormalVersion::V19);
        assert_eq!(NormalVersion::try_from(20).unwrap(), NormalVersion::V20);
        assert_eq!(NormalVersion::try_from(21).unwrap(), NormalVersion::V21);
        assert_eq!(NormalVersion::try_from(22).unwrap(), NormalVersion::V22);
        assert_eq!(NormalVersion::try_from(23).unwrap(), NormalVersion::V23);
        assert_eq!(NormalVersion::try_from(24).unwrap(), NormalVersion::V24);
        assert_eq!(NormalVersion::try_from(25).unwrap(), NormalVersion::V25);
        assert_eq!(NormalVersion::try_from(26).unwrap(), NormalVersion::V26);
        assert_eq!(NormalVersion::try_from(27).unwrap(), NormalVersion::V27);
        assert_eq!(NormalVersion::try_from(28).unwrap(), NormalVersion::V28);
        assert_eq!(NormalVersion::try_from(29).unwrap(), NormalVersion::V29);
        assert_eq!(NormalVersion::try_from(30).unwrap(), NormalVersion::V30);
        assert_eq!(NormalVersion::try_from(31).unwrap(), NormalVersion::V31);
        assert_eq!(NormalVersion::try_from(32).unwrap(), NormalVersion::V32);
        assert_eq!(NormalVersion::try_from(33).unwrap(), NormalVersion::V33);
        assert_eq!(NormalVersion::try_from(34).unwrap(), NormalVersion::V34);
        assert_eq!(NormalVersion::try_from(35).unwrap(), NormalVersion::V35);
        assert_eq!(NormalVersion::try_from(36).unwrap(), NormalVersion::V36);
        assert_eq!(NormalVersion::try_from(37).unwrap(), NormalVersion::V37);
        assert_eq!(NormalVersion::try_from(38).unwrap(), NormalVersion::V38);
        assert_eq!(NormalVersion::try_from(39).unwrap(), NormalVersion::V39);
        assert_eq!(NormalVersion::try_from(40).unwrap(), NormalVersion::V40);
    }

    #[test]
    fn try_from_u8_to_normal_version_with_invalid_version() {
        assert_eq!(
            NormalVersion::try_from(0).unwrap_err(),
            Error::InvalidVersion
        );
        assert_eq!(
            NormalVersion::try_from(41).unwrap_err(),
            Error::InvalidVersion
        );
        assert_eq!(
            NormalVersion::try_from(u8::MAX).unwrap_err(),
            Error::InvalidVersion
        );
    }
}
