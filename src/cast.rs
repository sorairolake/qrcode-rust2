// SPDX-FileCopyrightText: 2017 kennytm
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

pub trait Truncate {
    fn truncate_as_u8(self) -> u8;
}

impl Truncate for u16 {
    #[expect(clippy::cast_possible_truncation)]
    fn truncate_as_u8(self) -> u8 {
        self as u8
    }
}

#[expect(clippy::wrong_self_convention)]
pub trait As {
    fn as_i16(self) -> i16;
    fn as_isize(self) -> isize;
    fn as_u16(self) -> u16;
    fn as_u32(self) -> u32;
    fn as_usize(self) -> usize;
}

macro_rules! impl_as {
    ($ty:ty) => {
        #[cfg(debug_assertions)]
        impl As for $ty {
            fn as_i16(self) -> i16 {
                i16::try_from(self).unwrap()
            }

            fn as_isize(self) -> isize {
                isize::try_from(self).unwrap()
            }

            fn as_u16(self) -> u16 {
                u16::try_from(self).unwrap()
            }

            fn as_u32(self) -> u32 {
                u32::try_from(self).unwrap()
            }

            fn as_usize(self) -> usize {
                usize::try_from(self).unwrap()
            }
        }

        #[cfg(not(debug_assertions))]
        impl As for $ty {
            fn as_i16(self) -> i16 {
                self as i16
            }

            fn as_isize(self) -> isize {
                self as isize
            }

            fn as_u16(self) -> u16 {
                self as u16
            }

            fn as_u32(self) -> u32 {
                self as u32
            }

            fn as_usize(self) -> usize {
                self as usize
            }
        }
    };
}
impl_as!(i16);
impl_as!(isize);
impl_as!(u32);
impl_as!(usize);
