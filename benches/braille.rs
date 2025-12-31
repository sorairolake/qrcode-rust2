// SPDX-FileCopyrightText: 2025 Shun Sakai
// SPDX-FileCopyrightText: 2025 Mattias Jansson
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![feature(test)]

extern crate test;

use qrcode2::{QrCode, render::braille::BraillePixel};
use test::Bencher;

#[bench]
fn render_normal(b: &mut Bencher) {
    let code = QrCode::new(b"01234567").unwrap();
    b.iter(|| code.render::<BraillePixel>().build());
}

#[bench]
fn render_micro(b: &mut Bencher) {
    let code = QrCode::new_micro(b"01234567").unwrap();
    b.iter(|| code.render::<BraillePixel>().build());
}

#[bench]
fn render_rmqr(b: &mut Bencher) {
    let code = QrCode::new_rect_micro(b"01234567").unwrap();
    b.iter(|| code.render::<BraillePixel>().build());
}
