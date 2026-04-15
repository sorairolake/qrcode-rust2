// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg(feature = "eps")]

use qrcode2::{EcLevel, QrCode, Version, render::eps::Color};

#[test]
fn test_annex_i_qr_as_eps() {
    let code = QrCode::with_version(b"01234567", Version::Normal(1), EcLevel::M).unwrap();
    let image = code.render::<Color>().build();
    let expected = include_str!("data/test_annex_i_qr_as_eps.eps");
    assert_eq!(&image, expected);
}

#[test]
fn test_annex_i_micro_qr_as_eps() {
    let code = QrCode::with_version(b"01234567", Version::Micro(2), EcLevel::L).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(Color::new(0.5, 0.0, 0.0).unwrap())
        .light_color(Color::new(1.0, 1.0, 0.5).unwrap())
        .build();
    let expected = include_str!("data/test_annex_i_micro_qr_as_eps.eps");
    assert_eq!(&image, expected);
}

#[test]
fn test_annex_i_rmqr_as_eps() {
    let code = QrCode::with_version(b"0123456", Version::RectMicro(11, 27), EcLevel::H).unwrap();
    let image = code.render::<Color>().build();
    let expected = include_str!("data/test_annex_i_rmqr_as_eps.eps");
    assert_eq!(&image, expected);
}
