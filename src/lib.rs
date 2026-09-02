// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2016 Steven Allen
// SPDX-FileCopyrightText: 2019 Ivan Tham
// SPDX-FileCopyrightText: 2019 Jasper Bryant-Greene
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2024 Shun Sakai
// SPDX-FileCopyrightText: 2026 Lars Gerchow
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The `qrcode2` crate is a [QR code] encoding library.
//!
//! This crate provides a [QR code model 2], [Micro QR code], and [rMQR code]
//! encoder for binary data.
//!
//! # Examples
//!
//! ```
//! # #[cfg(feature = "image")]
//! # {
//! use qrcode2::{QrCode, image::Luma};
//!
//! // Encode some data into bits.
//! let code = QrCode::new(b"01234567").unwrap();
//!
//! // Render the bits into an image.
//! let image = code.render::<Luma<u8>>().build();
//!
//! // Save the image.
//! let temp_dir = tempfile::tempdir().unwrap();
//! image.save(temp_dir.path().join("qrcode.png")).unwrap();
//!
//! // You can also render it into a string.
//! let string = code.render().light_color(' ').dark_color('#').build();
//! println!("{string}");
//! # }
//! ```
//!
//! [QR code]: https://www.qrcode.com/
//! [QR code model 2]: https://www.qrcode.com/codes/model12.html
//! [Micro QR code]: https://www.qrcode.com/codes/microqr.html
//! [rMQR code]: https://www.qrcode.com/codes/rmqr.html

#![doc(html_root_url = "https://docs.rs/qrcode2/0.18.0/")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Lint levels of rustc.
#![deny(missing_docs)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod bits;
pub mod canvas;
mod cast;
pub mod ec;
pub mod error;
pub mod optimize;
pub mod render;
pub mod types;

use alloc::{string::String, vec::Vec};
use core::ops::Index;

#[cfg(feature = "svg")]
pub use csscolorparser;
#[cfg(feature = "image")]
pub use image;

use crate::{
    bits::{Bits, RectMicroStrategy},
    canvas::Canvas,
    cast::As,
    render::{Pixel, Renderer},
};
pub use crate::{
    error::{Error, Result},
    types::{Color, EcLevel, MicroVersion, NormalVersion, RectMicroVersion, Version},
};

/// The encoded QR code symbol.
#[derive(Clone, Debug)]
pub struct QrCode {
    content: Vec<Color>,
    version: Version,
    ec_level: EcLevel,
    width: usize,
    height: usize,
}

impl QrCode {
    /// Constructs a new QR code which automatically encodes the given data.
    ///
    /// This method uses [`EcLevel::M`] and automatically chooses the smallest
    /// QR code based on [`bits::encode_auto`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the QR code cannot be constructed, e.g. when the data
    /// is too long.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::QrCode;
    ///
    /// let code = QrCode::new(b"Some data").unwrap();
    /// ```
    pub fn new(data: impl AsRef<[u8]>) -> Result<Self> {
        Self::with_error_correction_level(data, EcLevel::default())
    }

    /// Constructs a new Micro QR code which automatically encodes the given
    /// data.
    ///
    /// This method uses [`EcLevel::M`] and automatically chooses the smallest
    /// Micro QR code based on [`bits::encode_auto_micro`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the Micro QR code cannot be constructed, e.g. when
    /// the data is too long.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::QrCode;
    ///
    /// let code = QrCode::new_micro(b"Some data").unwrap();
    /// ```
    pub fn new_micro(data: impl AsRef<[u8]>) -> Result<Self> {
        Self::micro_with_error_correction_level(data, EcLevel::default())
    }

    /// Constructs a new rMQR code which automatically encodes the given data.
    ///
    /// This method uses [`EcLevel::M`] and automatically chooses the smallest
    /// rMQR code based on [`bits::encode_auto_rect_micro`] and
    /// [`RectMicroStrategy::Area`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the rMQR code cannot be constructed, e.g. when the
    /// data is too long.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::QrCode;
    ///
    /// let code = QrCode::new_rect_micro(b"Some data").unwrap();
    /// ```
    pub fn new_rect_micro(data: impl AsRef<[u8]>) -> Result<Self> {
        Self::rect_micro_with_error_correction_level(data, EcLevel::default())
    }

    /// Constructs a new QR code which automatically encodes the given data at a
    /// specific error correction level.
    ///
    /// This method automatically chooses the smallest QR code based on
    /// [`bits::encode_auto`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the QR code cannot be constructed, e.g. when the data
    /// is too long.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{EcLevel, QrCode};
    ///
    /// let code = QrCode::with_error_correction_level(b"Some data", EcLevel::H).unwrap();
    /// ```
    pub fn with_error_correction_level(data: impl AsRef<[u8]>, ec_level: EcLevel) -> Result<Self> {
        let bits = bits::encode_auto(data.as_ref(), ec_level)?;
        Self::with_bits(bits, ec_level)
    }

    /// Constructs a new Micro QR code which automatically encodes the given
    /// data at a specific error correction level.
    ///
    /// This method automatically chooses the smallest Micro QR code based on
    /// [`bits::encode_auto_micro`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the Micro QR code cannot be constructed, e.g. when
    /// the data is too long.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{EcLevel, QrCode};
    ///
    /// let code = QrCode::micro_with_error_correction_level(b"Some data", EcLevel::Q).unwrap();
    /// ```
    pub fn micro_with_error_correction_level(
        data: impl AsRef<[u8]>,
        ec_level: EcLevel,
    ) -> Result<Self> {
        let bits = bits::encode_auto_micro(data.as_ref(), ec_level)?;
        Self::with_bits(bits, ec_level)
    }

    /// Constructs a new rMQR code which automatically encodes the given data at
    /// a specific error correction level.
    ///
    /// This method automatically chooses the smallest rMQR code based on
    /// [`bits::encode_auto_rect_micro`] and [`RectMicroStrategy::Area`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the rMQR code cannot be constructed, e.g. when the
    /// data is too long.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{EcLevel, QrCode};
    ///
    /// let code = QrCode::rect_micro_with_error_correction_level(b"Some data", EcLevel::H).unwrap();
    /// ```
    pub fn rect_micro_with_error_correction_level(
        data: impl AsRef<[u8]>,
        ec_level: EcLevel,
    ) -> Result<Self> {
        let bits = bits::encode_auto_rect_micro(data.as_ref(), ec_level, RectMicroStrategy::Area)?;
        Self::with_bits(bits, ec_level)
    }

    /// Constructs a new QR code for the given version and error correction
    /// level.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the QR code cannot be constructed, e.g. when the data
    /// is too long, or when the version and error correction level are
    /// incompatible.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{EcLevel, NormalVersion, QrCode, Version};
    ///
    /// let code =
    ///     QrCode::with_version(b"Some data", Version::Normal(NormalVersion::V5), EcLevel::M).unwrap();
    /// ```
    ///
    /// This method can also be used to generate Micro QR code or rMQR code.
    ///
    /// ```
    /// use qrcode2::{EcLevel, MicroVersion, QrCode, RectMicroVersion, Version};
    ///
    /// let micro_code =
    ///     QrCode::with_version(b"123", Version::Micro(MicroVersion::M1), EcLevel::L).unwrap();
    /// let rmqr_code = QrCode::with_version(
    ///     b"456",
    ///     Version::RectMicro(RectMicroVersion::R7x43),
    ///     EcLevel::M,
    /// )
    /// .unwrap();
    /// ```
    pub fn with_version(
        data: impl AsRef<[u8]>,
        version: Version,
        ec_level: EcLevel,
    ) -> Result<Self> {
        let mut bits = Bits::new(version);
        bits.push_optimal_data(data.as_ref())?;
        bits.push_terminator(ec_level)?;
        Self::with_bits(bits, ec_level)
    }

    /// Constructs a new QR code with encoded bits.
    ///
    /// Use this method only if there are very special need to manipulate the
    /// raw bits before encoding. Some examples are:
    ///
    /// - Encode data using specific character set with ECI.
    /// - Use the FNC1 modes.
    /// - Avoid the optimal segmentation algorithm.
    ///
    /// See the [`Bits`] structure for detail.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the QR code cannot be constructed, e.g. when the bits
    /// are too long, or when the version and error correction level are
    /// incompatible.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{EcLevel, NormalVersion, QrCode, Version, bits::Bits};
    ///
    /// let mut bits = Bits::new(Version::Normal(NormalVersion::V1));
    /// bits.push_eci_designator(9);
    /// bits.push_byte_data(b"\xCA\xFE\xE4\xE9\xEA\xE1\xF2 QR");
    /// bits.push_terminator(EcLevel::L);
    /// let qrcode = QrCode::with_bits(bits, EcLevel::L);
    /// ```
    pub fn with_bits(bits: Bits, ec_level: EcLevel) -> Result<Self> {
        let version = bits.version();
        let data = bits.into_bytes();
        let (encoded_data, ec_data) = ec::construct_codewords(&data, version, ec_level)?;
        let mut canvas = Canvas::new(version, ec_level);
        canvas.draw_all_functional_patterns();
        canvas.draw_data(&encoded_data, &ec_data);
        let content = canvas.apply_best_mask().into_colors();
        let (width, height) = (version.width().as_usize(), version.height().as_usize());
        Ok(Self {
            content,
            version,
            ec_level,
            width,
            height,
        })
    }

    /// Gets the version of this QR code.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{NormalVersion, QrCode, Version};
    ///
    /// let code = QrCode::new(b"Some data").unwrap();
    /// assert_eq!(code.version(), Version::Normal(NormalVersion::V1));
    /// ```
    #[must_use]
    pub const fn version(&self) -> Version {
        self.version
    }

    /// Gets the error correction level of this QR code.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::{EcLevel, QrCode};
    ///
    /// let code = QrCode::new(b"Some data").unwrap();
    /// assert_eq!(code.error_correction_level(), EcLevel::M);
    /// ```
    #[must_use]
    pub const fn error_correction_level(&self) -> EcLevel {
        self.ec_level
    }

    /// Gets the number of modules per side, i.e. the width of this QR code.
    ///
    /// The width here does not contain the quiet zone paddings.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::QrCode;
    ///
    /// let code = QrCode::new_rect_micro(b"Some data").unwrap();
    /// assert_eq!(code.width(), 27);
    /// ```
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Gets the number of modules per side, i.e. the height of this QR code.
    ///
    /// The height here does not contain the quiet zone paddings.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::QrCode;
    ///
    /// let code = QrCode::new_rect_micro(b"Some data").unwrap();
    /// assert_eq!(code.height(), 13);
    /// ```
    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    #[expect(clippy::missing_panics_doc)]
    /// Gets the maximum number of allowed erratic modules can be introduced
    /// before the data becomes corrupted. Note that errors should not be
    /// introduced to functional modules.
    ///
    /// # Examples
    ///
    /// ```
    /// use qrcode2::QrCode;
    ///
    /// let code = QrCode::new(b"Some data").unwrap();
    /// assert_eq!(code.max_allowed_errors(), 4);
    /// ```
    #[must_use]
    pub fn max_allowed_errors(&self) -> usize {
        ec::max_allowed_errors(self.version, self.ec_level).unwrap()
    }

    /// Checks whether a module at coordinate (x, y) is a functional module or
    /// not.
    ///
    /// # Panics
    ///
    /// Panics if `x` or `y` is beyond the size of the QR code.
    #[must_use]
    pub fn is_functional(&self, x: usize, y: usize) -> bool {
        let x = x.try_into().unwrap();
        let y = y.try_into().unwrap();
        canvas::is_functional(self.version, self.version.width(), x, y)
    }

    /// Converts the QR code into a human-readable string. This is mainly for
    /// debugging only.
    #[must_use]
    pub fn to_debug_str(&self, on_char: char, off_char: char) -> String {
        self.render()
            .has_quiet_zone(false)
            .dark_color(on_char)
            .light_color(off_char)
            .build()
    }

    /// Converts the QR code to a vector of colors.
    #[must_use]
    pub fn to_colors(&self) -> Vec<Color> {
        self.content.clone()
    }

    /// Consumes the QR code, returning a vector of colors.
    #[must_use]
    pub fn into_colors(self) -> Vec<Color> {
        self.content
    }

    /// Renders the QR code into an image. The result is an image builder, which
    /// you may do some additional configuration before copying it into a
    /// concrete image.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "image")]
    /// # {
    /// use qrcode2::{
    ///     QrCode,
    ///     image::{Rgb, imageops},
    /// };
    ///
    /// let mut image = QrCode::new(b"hello")
    ///     .unwrap()
    ///     .render::<Rgb<u8>>()
    ///     .dark_color(Rgb([0, 0, 128]))
    ///     .light_color(Rgb([224, 224, 224]))
    ///     .has_quiet_zone(false)
    ///     .min_dimensions(300, 300)
    ///     .build();
    ///
    /// // Flip the QR code vertically.
    /// imageops::rotate180_in_place(&mut image);
    /// let temp_dir = tempfile::tempdir().unwrap();
    /// image.save(temp_dir.path().join("qrcode.png")).unwrap();
    /// # }
    /// ```
    #[must_use]
    pub fn render<P: Pixel>(&self) -> Renderer<'_, P> {
        let quiet_zone = if self.version.is_normal() { 4 } else { 2 };
        Renderer::new(&self.content, self.width, self.height, quiet_zone)
    }
}

impl Index<(usize, usize)> for QrCode {
    type Output = Color;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        let index = y * self.width + x;
        &self.content[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annex_i_qr() {
        // This uses the ISO Annex I as test vector.
        let code =
            QrCode::with_version(b"01234567", Version::Normal(NormalVersion::V1), EcLevel::M)
                .unwrap();
        assert_eq!(
            code.to_debug_str('#', '.'),
            concat!(
                "#######..#.##.#######\n",
                "#.....#..####.#.....#\n",
                "#.###.#.#.....#.###.#\n",
                "#.###.#.##....#.###.#\n",
                "#.###.#.#.###.#.###.#\n",
                "#.....#.#...#.#.....#\n",
                "#######.#.#.#.#######\n",
                "........#..##........\n",
                "#.#####..#..#.#####..\n",
                "...#.#.##.#.#..#.##..\n",
                "..#...##.#.#.#..#####\n",
                "....#....#.....####..\n",
                "...######..#.#..#....\n",
                "........#.#####..##..\n",
                "#######..##.#.##.....\n",
                "#.....#.#.#####...#.#\n",
                "#.###.#.#...#..#.##..\n",
                "#.###.#.##..#..#.....\n",
                "#.###.#.#.##.#..#.#..\n",
                "#.....#........##.##.\n",
                "#######.####.#..#.#.."
            )
        );
    }

    #[test]
    fn annex_i_micro_qr() {
        let code = QrCode::with_version(b"01234567", Version::Micro(MicroVersion::M2), EcLevel::L)
            .unwrap();
        assert_eq!(
            code.to_debug_str('#', '.'),
            concat!(
                "#######.#.#.#\n",
                "#.....#.###.#\n",
                "#.###.#..##.#\n",
                "#.###.#..####\n",
                "#.###.#.###..\n",
                "#.....#.#...#\n",
                "#######..####\n",
                ".........##..\n",
                "##.#....#...#\n",
                ".##.#.#.#.#.#\n",
                "###..#######.\n",
                "...#.#....##.\n",
                "###.#..##.###"
            )
        );
    }

    #[test]
    fn annex_i_rmqr() {
        let code = QrCode::with_version(
            b"0123456",
            Version::RectMicro(RectMicroVersion::R11x27),
            EcLevel::H,
        )
        .unwrap();
        assert_eq!(
            code.to_debug_str('#', '.'),
            concat!(
                "#######.#.#.#.#.#.#.#.#.###\n",
                "#.....#..##.#....###.#..#.#\n",
                "#.###.#....#..####.#..#####\n",
                "#.###.#.####.##.####...##..\n",
                "#.###.#..#.###.######..#.##\n",
                "#.....#.###....#..#####..#.\n",
                "#######.#########...#.#####\n",
                "........#####.#.##.#..#...#\n",
                "####......#..#.#..#####.#.#\n",
                "#.#.#.#..##..#.#..###.#...#\n",
                "###.#.#.#.#.#.#.#.#.#.#####"
            )
        );
    }
}

#[cfg(test)]
mod iso_capacity {
    //! Data-driven conformance for QR character capacities (Micro, Normal,
    //! rMQR).
    //!
    //! The encoder validates against the bit-capacity table `DATA_LENGTHS`
    //! (ISO/IEC 18004:2006 Table 7, ISO/IEC 23941:2022 Table 6); the per-mode
    //! *character* capacities below are derived from it at runtime and are not
    //! otherwise asserted anywhere. These tests lock both the boundary (exactly
    //! `cap` chars encode, `cap + 1` does not) and the segmentation that
    //! determines whether mixed-mode payloads reach that boundary at all.

    use alloc::{vec, vec::Vec};

    use super::*;

    /// One char that forces each mode: numeric, alphanumeric (non-numeric so it
    /// can't be re-encoded as numeric), and byte (lowercase, byte-only).
    const MODE_UNITS: [u8; 3] = [b'7', b'A', b'a'];

    /// `(version, ec, [numeric, alphanumeric, byte])` character capacities.
    /// Micro and Normal QR are from ISO/IEC 18004:2006; rMQR from ISO/IEC
    /// 23941:2022. Kanji is omitted (it needs Shift-JIS fixtures). `None` marks
    /// a mode/EC level not valid for the version. The clamp fix applies to
    /// every version type, so the boundary is asserted across all three
    /// families.
    const CAPACITIES: &[(Version, EcLevel, [Option<usize>; 3])] = &[
        // Micro QR
        (
            Version::Micro(MicroVersion::M1),
            EcLevel::L,
            [Some(5), None, None],
        ),
        (
            Version::Micro(MicroVersion::M2),
            EcLevel::L,
            [Some(10), Some(6), None],
        ),
        (
            Version::Micro(MicroVersion::M2),
            EcLevel::M,
            [Some(8), Some(5), None],
        ),
        (
            Version::Micro(MicroVersion::M3),
            EcLevel::L,
            [Some(23), Some(14), Some(9)],
        ),
        (
            Version::Micro(MicroVersion::M3),
            EcLevel::M,
            [Some(18), Some(11), Some(7)],
        ),
        (
            Version::Micro(MicroVersion::M4),
            EcLevel::L,
            [Some(35), Some(21), Some(15)],
        ),
        (
            Version::Micro(MicroVersion::M4),
            EcLevel::M,
            [Some(30), Some(18), Some(13)],
        ),
        (
            Version::Micro(MicroVersion::M4),
            EcLevel::Q,
            [Some(21), Some(13), Some(9)],
        ),
        // Normal QR, Version 1 (all four EC levels)
        (
            Version::Normal(NormalVersion::V1),
            EcLevel::L,
            [Some(41), Some(25), Some(17)],
        ),
        (
            Version::Normal(NormalVersion::V1),
            EcLevel::M,
            [Some(34), Some(20), Some(14)],
        ),
        (
            Version::Normal(NormalVersion::V1),
            EcLevel::Q,
            [Some(27), Some(16), Some(11)],
        ),
        (
            Version::Normal(NormalVersion::V1),
            EcLevel::H,
            [Some(17), Some(10), Some(7)],
        ),
        // rMQR (M and H only)
        (
            Version::RectMicro(RectMicroVersion::R7x43),
            EcLevel::M,
            [Some(12), Some(7), Some(5)],
        ),
        (
            Version::RectMicro(RectMicroVersion::R7x43),
            EcLevel::H,
            [Some(5), Some(3), Some(2)],
        ),
        (
            Version::RectMicro(RectMicroVersion::R11x27),
            EcLevel::M,
            [Some(14), Some(8), Some(6)],
        ),
        (
            Version::RectMicro(RectMicroVersion::R11x27),
            EcLevel::H,
            [Some(9), Some(6), Some(4)],
        ),
    ];

    fn fill(unit: u8, n: usize) -> Vec<u8> {
        vec![unit; n]
    }

    #[test]
    fn capacity_boundaries() {
        for &(version, ec, caps) in CAPACITIES {
            for (mode_idx, cap) in caps.iter().enumerate() {
                let Some(cap) = *cap else { continue };
                let unit = MODE_UNITS[mode_idx];
                assert!(
                    QrCode::with_version(fill(unit, cap), version, ec).is_ok(),
                    "{version:?} {ec:?}: {cap} of {:?} should fit",
                    unit as char
                );
                assert!(
                    QrCode::with_version(fill(unit, cap + 1), version, ec).is_err(),
                    "{version:?} {ec:?}: {} of {:?} should overflow",
                    cap + 1,
                    unit as char
                );
            }
        }
    }

    /// Mixed-mode payloads at the M3-L boundary: each is 14 QR-alphanumeric
    /// characters (fits as one alphanumeric segment, 83 <= 84 bits) but
    /// contains internal digit runs that the greedy optimizer used to split
    /// into separate numeric segments, overrunning capacity and falsely
    /// rejecting the input.
    #[test]
    fn micro_m3_mixed_mode_vectors() {
        const FIT: &[&[u8]] = &[
            b"9BA3935DM3TBE4",
            b"AB3935CD12EF99",
            b"1234ABCD5678EF",
            b"ZX99887766XYZ5",
        ];
        for &payload in FIT {
            assert_eq!(payload.len(), 14);
            assert!(
                QrCode::with_version(payload, Version::Micro(MicroVersion::M3), EcLevel::L).is_ok(),
                "{:?} should fit M3-L as a single alphanumeric segment",
                core::str::from_utf8(payload).unwrap()
            );
        }
    }
}
