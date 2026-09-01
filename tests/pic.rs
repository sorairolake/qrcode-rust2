// SPDX-FileCopyrightText: 2024 Alexis Hildebrandt
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg(feature = "pic")]

use qrcode2::{
    EcLevel, MicroVersion, NormalVersion, QrCode, RectMicroVersion, Version, render::pic::Color,
};

#[test]
fn annex_i_qr_as_pic() {
    let code = QrCode::with_version(b"01234567", Version::Normal(NormalVersion::V1), EcLevel::M).unwrap();
    let image = code.render::<Color>().build();
    let expected = include_str!("data/test_annex_i_qr_as_pic.pic");
    assert_eq!(&image, expected);
}

#[test]
fn annex_i_micro_qr_as_pic() {
    let code = QrCode::with_version(b"01234567", Version::Micro(MicroVersion::M2), EcLevel::L).unwrap();
    let image = code.render::<Color>().min_dimensions(1, 1).build();
    let expected = include_str!("data/test_annex_i_micro_qr_as_pic.pic");
    assert_eq!(&image, expected);
}

#[test]
fn annex_i_rmqr_as_pic() {
    let code = QrCode::with_version(b"0123456", Version::RectMicro(RectMicroVersion::R11x27), EcLevel::H).unwrap();
    let image = code.render::<Color>().build();
    let expected = include_str!("data/test_annex_i_rmqr_as_pic.pic");
    assert_eq!(&image, expected);
}
