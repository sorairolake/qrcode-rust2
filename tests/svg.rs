// SPDX-FileCopyrightText: 2017 kennytm
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg(feature = "svg")]

use qrcode2::{
    EcLevel, MicroVersion, NormalVersion, QrCode, RectMicroVersion, Version, render::svg::Color,
};

#[test]
fn annex_i_qr_as_svg() {
    let code =
        QrCode::with_version(b"01234567", Version::Normal(NormalVersion::V1), EcLevel::M).unwrap();
    let image = code.render::<Color<'_>>().build();
    let expected = include_str!("data/annex_i_qr_as_svg.svg");
    assert_eq!(&image, expected);
}

#[test]
fn annex_i_micro_qr_as_svg() {
    let code =
        QrCode::with_version(b"01234567", Version::Micro(MicroVersion::M2), EcLevel::L).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(Color::new("#800000").unwrap())
        .light_color(Color::new("#ffff80").unwrap())
        .build();
    let expected = include_str!("data/annex_i_micro_qr_as_svg.svg");
    assert_eq!(&image, expected);
}

#[test]
fn rmqr_as_svg() {
    let code = QrCode::with_version(
        b"0123456",
        Version::RectMicro(RectMicroVersion::R11x27),
        EcLevel::H,
    )
    .unwrap();
    let image = code.render::<Color<'_>>().build();
    let expected = include_str!("data/rmqr_as_svg.svg");
    assert_eq!(&image, expected);
}

#[test]
fn annex_i_micro_qr_as_svg_named_color() {
    let code =
        QrCode::with_version(b"01234567", Version::Micro(MicroVersion::M2), EcLevel::L).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(Color::new("brown").unwrap())
        .light_color(Color::new("white").unwrap())
        .build();
    let expected = include_str!("data/annex_i_micro_qr_as_svg_named_color.svg");
    assert_eq!(&image, expected);
}

#[test]
fn annex_i_micro_qr_as_svg_rgb() {
    let code =
        QrCode::with_version(b"01234567", Version::Micro(MicroVersion::M2), EcLevel::L).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(Color::new("rgb(165 42 42)").unwrap())
        .light_color(Color::new("rgb(255 255 255)").unwrap())
        .build();
    let expected = include_str!("data/annex_i_micro_qr_as_svg_rgb.svg");
    assert_eq!(&image, expected);
}

#[test]
fn annex_i_micro_qr_as_svg_hsl() {
    let code =
        QrCode::with_version(b"01234567", Version::Micro(MicroVersion::M2), EcLevel::L).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(Color::new("hsl(248 39% 39.2%)").unwrap())
        .light_color(Color::new("hsl(0 0% 100%)").unwrap())
        .build();
    let expected = include_str!("data/annex_i_micro_qr_as_svg_hsl.svg");
    assert_eq!(&image, expected);
}

#[test]
fn annex_i_micro_qr_as_svg_hwb() {
    let code =
        QrCode::with_version(b"01234567", Version::Micro(MicroVersion::M2), EcLevel::L).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(Color::new("hwb(50.6 0% 0%)").unwrap())
        .light_color(Color::new("hwb(none 100% 0%)").unwrap())
        .build();
    let expected = include_str!("data/annex_i_micro_qr_as_svg_hwb.svg");
    assert_eq!(&image, expected);
}

#[test]
fn annex_i_micro_qr_as_svg_lab() {
    let code =
        QrCode::with_version(b"01234567", Version::Micro(MicroVersion::M2), EcLevel::L).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(Color::new("lab(42.5% -21 -20.7)").unwrap())
        .light_color(Color::new("lab(100% 0 0)").unwrap())
        .build();
    let expected = include_str!("data/annex_i_micro_qr_as_svg_lab.svg");
    assert_eq!(&image, expected);
}

#[test]
fn annex_i_micro_qr_as_svg_lch() {
    let code =
        QrCode::with_version(b"01234567", Version::Micro(MicroVersion::M2), EcLevel::L).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(Color::new("lch(45.5% 69 3.1)").unwrap())
        .light_color(Color::new("lch(100% 0 0)").unwrap())
        .build();
    let expected = include_str!("data/annex_i_micro_qr_as_svg_lch.svg");
    assert_eq!(&image, expected);
}

#[test]
fn annex_i_micro_qr_as_svg_oklab() {
    let code =
        QrCode::with_version(b"01234567", Version::Micro(MicroVersion::M2), EcLevel::L).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(Color::new("oklab(50.4% -0.0906 0.0069)").unwrap())
        .light_color(Color::new("oklab(100% 0 0)").unwrap())
        .build();
    let expected = include_str!("data/annex_i_micro_qr_as_svg_oklab.svg");
    assert_eq!(&image, expected);
}

#[test]
fn annex_i_micro_qr_as_svg_oklch() {
    let code =
        QrCode::with_version(b"01234567", Version::Micro(MicroVersion::M2), EcLevel::L).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(Color::new("oklch(59.41% 0.16 301.29)").unwrap())
        .light_color(Color::new("oklch(100% 0 0)").unwrap())
        .build();
    let expected = include_str!("data/annex_i_micro_qr_as_svg_oklch.svg");
    assert_eq!(&image, expected);
}
