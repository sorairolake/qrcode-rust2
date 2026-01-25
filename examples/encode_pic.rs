// SPDX-FileCopyrightText: 2024 Alexis Hildebrandt
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::Context;
use clap::{Parser, ValueEnum};
use qrcode2::{EcLevel, QrCode, Version, render::pic::Color};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Opt {
    /// Error correction level.
    #[arg(
        short('l'),
        long,
        value_enum,
        default_value_t,
        value_name("LEVEL"),
        ignore_case(true)
    )]
    error_correction_level: Ecc,

    /// The version of the symbol.
    #[arg(short('v'), long, num_args(1..=2), value_name("NUMBER"))]
    symbol_version: Option<Vec<i16>>,

    /// The type of QR code.
    #[arg(
        long,
        value_enum,
        default_value_t,
        value_name("TYPE"),
        ignore_case(true)
    )]
    variant: Variant,

    /// Input data.
    string: String,
}

#[derive(Clone, Debug, Default, ValueEnum)]
enum Ecc {
    /// Level L.
    L,

    /// Level M.
    #[default]
    M,

    /// Level Q.
    Q,

    /// Level H.
    H,
}

impl From<Ecc> for EcLevel {
    fn from(level: Ecc) -> Self {
        match level {
            Ecc::L => Self::L,
            Ecc::M => Self::M,
            Ecc::Q => Self::Q,
            Ecc::H => Self::H,
        }
    }
}

#[derive(Clone, Debug, Default, ValueEnum)]
enum Variant {
    /// Normal QR code.
    #[default]
    Normal,

    /// Micro QR code.
    Micro,

    /// rMQR code.
    Rmqr,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    let input = opt.string;
    let ec_level = opt.error_correction_level.into();
    let code = if let Some(sv) = opt.symbol_version {
        let version = match opt.variant {
            Variant::Normal => Version::Normal(sv[0]),
            Variant::Micro => Version::Micro(sv[0]),
            Variant::Rmqr => Version::RectMicro(sv[0], sv[1]),
        };
        QrCode::with_version(input, version, ec_level)
    } else {
        match opt.variant {
            Variant::Normal => QrCode::with_error_correction_level(input, ec_level),
            Variant::Micro => QrCode::micro_with_error_correction_level(input, ec_level),
            Variant::Rmqr => QrCode::rect_micro_with_error_correction_level(input, ec_level),
        }
    }
    .context("could not construct a QR code")?;

    let image = code.render::<Color>().build();

    println!("{image}");
    Ok(())
}
