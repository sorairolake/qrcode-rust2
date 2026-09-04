// SPDX-FileCopyrightText: 2016 kennytm
// SPDX-FileCopyrightText: 2019 Jasper Bryant-Greene
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg(feature = "image")]

use std::sync::LazyLock;

use qrcode2::{
    EcLevel, Error, MicroVersion, NormalVersion, QrCode, RectMicroVersion, Version,
    image::{Luma, Rgb},
};
use shake::{ExtendableOutput, Shake128};

static INPUT_DATA: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let mut buf = vec![u8::default(); 2954];
    Shake128::digest_xof(b"01234567", &mut buf);
    buf
});

#[test]
fn annex_i_qr_as_image() {
    let code =
        QrCode::with_version(b"01234567", Version::Normal(NormalVersion::V1), EcLevel::M).unwrap();
    let image = code.render::<Luma<u8>>().build();
    let expected = image::load_from_memory(include_bytes!("data/annex_i_qr_as_image.png"))
        .unwrap()
        .into_luma8();
    assert_eq!(image.dimensions(), expected.dimensions());
    assert_eq!(image.into_raw(), expected.into_raw());
}

#[test]
fn annex_i_micro_qr_as_image() {
    let code =
        QrCode::with_version(b"01234567", Version::Micro(MicroVersion::M2), EcLevel::L).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(Rgb([128, 0, 0]))
        .light_color(Rgb([255, 255, 128]))
        .build();
    let expected = image::load_from_memory(include_bytes!("data/annex_i_micro_qr_as_image.png"))
        .unwrap()
        .into_rgb8();
    assert_eq!(image.dimensions(), expected.dimensions());
    assert_eq!(image.into_raw(), expected.into_raw());
}

#[test]
fn rmqr_as_image() {
    let code = QrCode::with_version(
        b"0123456",
        Version::RectMicro(RectMicroVersion::R11x27),
        EcLevel::H,
    )
    .unwrap();
    let image = code.render::<Luma<u8>>().build();
    let expected = image::load_from_memory(include_bytes!("data/rmqr_as_image.png"))
        .unwrap()
        .into_luma8();
    assert_eq!(image.dimensions(), expected.dimensions());
    assert_eq!(image.into_raw(), expected.into_raw());
}

#[test]
fn qr_v40_ec_l_as_image() {
    {
        let code = QrCode::with_error_correction_level(&INPUT_DATA[..2953], EcLevel::L).unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected = image::load_from_memory(include_bytes!("data/qr_v40_ec_l_as_image.png"))
            .unwrap()
            .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::with_error_correction_level(&*INPUT_DATA, EcLevel::L).unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
    {
        let code = QrCode::with_version(
            &INPUT_DATA[..2953],
            Version::Normal(NormalVersion::V40),
            EcLevel::L,
        )
        .unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected = image::load_from_memory(include_bytes!("data/qr_v40_ec_l_as_image.png"))
            .unwrap()
            .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::with_version(
            &*INPUT_DATA,
            Version::Normal(NormalVersion::V40),
            EcLevel::L,
        )
        .unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
}

#[test]
fn qr_v40_ec_m_as_image() {
    {
        let code = QrCode::new(&INPUT_DATA[..2331]).unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected = image::load_from_memory(include_bytes!("data/qr_v40_ec_m_as_image.png"))
            .unwrap()
            .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::new(&INPUT_DATA[..2332]).unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
    {
        let code = QrCode::with_error_correction_level(&INPUT_DATA[..2331], EcLevel::M).unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected = image::load_from_memory(include_bytes!("data/qr_v40_ec_m_as_image.png"))
            .unwrap()
            .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::with_error_correction_level(&INPUT_DATA[..2332], EcLevel::M).unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
    {
        let code = QrCode::with_version(
            &INPUT_DATA[..2331],
            Version::Normal(NormalVersion::V40),
            EcLevel::M,
        )
        .unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected = image::load_from_memory(include_bytes!("data/qr_v40_ec_m_as_image.png"))
            .unwrap()
            .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::with_version(
            &INPUT_DATA[..2332],
            Version::Normal(NormalVersion::V40),
            EcLevel::M,
        )
        .unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
}

#[test]
fn qr_v40_ec_h_as_image() {
    {
        let code = QrCode::with_error_correction_level(&INPUT_DATA[..1273], EcLevel::H).unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected = image::load_from_memory(include_bytes!("data/qr_v40_ec_h_as_image.png"))
            .unwrap()
            .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::with_error_correction_level(&INPUT_DATA[..1274], EcLevel::H).unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
    {
        let code = QrCode::with_version(
            &INPUT_DATA[..1273],
            Version::Normal(NormalVersion::V40),
            EcLevel::H,
        )
        .unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected = image::load_from_memory(include_bytes!("data/qr_v40_ec_h_as_image.png"))
            .unwrap()
            .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::with_version(
            &INPUT_DATA[..1274],
            Version::Normal(NormalVersion::V40),
            EcLevel::H,
        )
        .unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
}

#[test]
fn micro_qr_m4_ec_l_as_image() {
    {
        let code =
            QrCode::micro_with_error_correction_level(&INPUT_DATA[..15], EcLevel::L).unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected =
            image::load_from_memory(include_bytes!("data/micro_qr_m4_ec_l_as_image.png"))
                .unwrap()
                .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err =
            QrCode::micro_with_error_correction_level(&INPUT_DATA[..16], EcLevel::L).unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
    {
        let code = QrCode::with_version(
            &INPUT_DATA[..15],
            Version::Micro(MicroVersion::M4),
            EcLevel::L,
        )
        .unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected =
            image::load_from_memory(include_bytes!("data/micro_qr_m4_ec_l_as_image.png"))
                .unwrap()
                .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::with_version(
            &INPUT_DATA[..16],
            Version::Micro(MicroVersion::M4),
            EcLevel::L,
        )
        .unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
}

#[test]
fn micro_qr_m4_ec_m_as_image() {
    {
        let code = QrCode::new_micro(&INPUT_DATA[..13]).unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected =
            image::load_from_memory(include_bytes!("data/micro_qr_m4_ec_m_as_image.png"))
                .unwrap()
                .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::new_micro(&INPUT_DATA[..14]).unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
    {
        let code =
            QrCode::micro_with_error_correction_level(&INPUT_DATA[..13], EcLevel::M).unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected =
            image::load_from_memory(include_bytes!("data/micro_qr_m4_ec_m_as_image.png"))
                .unwrap()
                .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err =
            QrCode::micro_with_error_correction_level(&INPUT_DATA[..14], EcLevel::M).unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
    {
        let code = QrCode::with_version(
            &INPUT_DATA[..13],
            Version::Micro(MicroVersion::M4),
            EcLevel::M,
        )
        .unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected =
            image::load_from_memory(include_bytes!("data/micro_qr_m4_ec_m_as_image.png"))
                .unwrap()
                .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::with_version(
            &INPUT_DATA[..14],
            Version::Micro(MicroVersion::M4),
            EcLevel::M,
        )
        .unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
}

#[test]
fn micro_qr_m4_ec_q_as_image() {
    {
        let code = QrCode::micro_with_error_correction_level(&INPUT_DATA[..9], EcLevel::Q).unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected =
            image::load_from_memory(include_bytes!("data/micro_qr_m4_ec_q_as_image.png"))
                .unwrap()
                .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err =
            QrCode::micro_with_error_correction_level(&INPUT_DATA[..10], EcLevel::Q).unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
    {
        let code = QrCode::with_version(
            &INPUT_DATA[..9],
            Version::Micro(MicroVersion::M4),
            EcLevel::Q,
        )
        .unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected =
            image::load_from_memory(include_bytes!("data/micro_qr_m4_ec_q_as_image.png"))
                .unwrap()
                .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::with_version(
            &INPUT_DATA[..10],
            Version::Micro(MicroVersion::M4),
            EcLevel::Q,
        )
        .unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
}

#[test]
fn rmqr_r17x139_ec_m_as_image() {
    {
        let code = QrCode::new_rect_micro(&INPUT_DATA[..150]).unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected =
            image::load_from_memory(include_bytes!("data/rmqr_r17x139_ec_m_as_image.png"))
                .unwrap()
                .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::new_rect_micro(&INPUT_DATA[..151]).unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
    {
        let code =
            QrCode::rect_micro_with_error_correction_level(&INPUT_DATA[..150], EcLevel::M).unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected =
            image::load_from_memory(include_bytes!("data/rmqr_r17x139_ec_m_as_image.png"))
                .unwrap()
                .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::rect_micro_with_error_correction_level(&INPUT_DATA[..151], EcLevel::M)
            .unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
    {
        let code = QrCode::with_version(
            &INPUT_DATA[..150],
            Version::RectMicro(RectMicroVersion::R17x139),
            EcLevel::M,
        )
        .unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected =
            image::load_from_memory(include_bytes!("data/rmqr_r17x139_ec_m_as_image.png"))
                .unwrap()
                .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::with_version(
            &INPUT_DATA[..151],
            Version::RectMicro(RectMicroVersion::R17x139),
            EcLevel::M,
        )
        .unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
}

#[test]
fn rmqr_r17x139_ec_h_as_image() {
    {
        let code =
            QrCode::rect_micro_with_error_correction_level(&INPUT_DATA[..74], EcLevel::H).unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected =
            image::load_from_memory(include_bytes!("data/rmqr_r17x139_ec_h_as_image.png"))
                .unwrap()
                .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::rect_micro_with_error_correction_level(&INPUT_DATA[..75], EcLevel::H)
            .unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
    {
        let code = QrCode::with_version(
            &INPUT_DATA[..74],
            Version::RectMicro(RectMicroVersion::R17x139),
            EcLevel::H,
        )
        .unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected =
            image::load_from_memory(include_bytes!("data/rmqr_r17x139_ec_h_as_image.png"))
                .unwrap()
                .into_luma8();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
    {
        let err = QrCode::with_version(
            &INPUT_DATA[..75],
            Version::RectMicro(RectMicroVersion::R17x139),
            EcLevel::H,
        )
        .unwrap_err();
        assert_eq!(err, Error::DataTooLong);
    }
}
