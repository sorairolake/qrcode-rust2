// SPDX-FileCopyrightText: 2017 kennytm
// SPDX-FileCopyrightText: 2024 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! An example of encoding a string into a QR code and outputting it as a
//! string.

use anyhow::Context;
use clap::{Parser, ValueEnum};
use qrcode2::{EcLevel, MicroVersion, NormalVersion, QrCode, RectMicroVersion, Version};

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
    symbol_version: Option<Vec<u8>>,

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
    /// QR code.
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
            Variant::Normal => {
                let version = NormalVersion::try_from(sv[0])
                    .context("could not set a QR code version")?;
                Version::Normal(version)
            }
            Variant::Micro => {
                let version = MicroVersion::try_from(sv[0])
                    .context("could not set a Micro QR code version")?;
                Version::Micro(version)
            }
            Variant::Rmqr => {
                let version = RectMicroVersion::try_from((sv[0], sv[1]))
                    .context("could not set a rMQR code version")?;
                Version::RectMicro(version)
            }
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

    let image = code.render::<char>().module_dimensions(2, 1).build();

    println!("{image}");
    Ok(())
}
