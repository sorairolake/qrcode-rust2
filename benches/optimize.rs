// SPDX-FileCopyrightText: 2026 Lars Gerchow
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![feature(test)]

extern crate test;

use qrcode2::{
    Version,
    optimize::{self, Parser},
};
use test::{Bencher, black_box};

const VERSION: Version = Version::Normal(40);

/// Worst case for the run-boundary DP: every character is its own run, so the
/// run count `k` equals the input length `n` and the DP does `O(k^2)` work.
fn alternating(n: usize) -> Vec<u8> {
    (0..n)
        .map(|i| if i % 2 == 0 { b'A' } else { b'1' })
        .collect()
}

fn bench_dp(b: &mut Bencher, data: &[u8]) {
    b.iter(|| optimize::optimize(black_box(data), VERSION));
}

fn bench_greedy(b: &mut Bencher, data: &[u8]) {
    b.iter(|| {
        Parser::new(black_box(data))
            .optimize(VERSION)
            .collect::<Vec<_>>()
    });
}

// --- Optimal DP, worst-case alternating input (k == n) -----------------------

#[bench]
fn dp_alternating_64(b: &mut Bencher) {
    bench_dp(b, &alternating(64));
}

#[bench]
fn dp_alternating_128(b: &mut Bencher) {
    bench_dp(b, &alternating(128));
}

#[bench]
fn dp_alternating_256(b: &mut Bencher) {
    bench_dp(b, &alternating(256));
}

#[bench]
fn dp_alternating_512(b: &mut Bencher) {
    bench_dp(b, &alternating(512));
}

#[bench]
fn dp_alternating_1024(b: &mut Bencher) {
    bench_dp(b, &alternating(1024));
}

// --- Greedy, same inputs (linear baseline for comparison) --------------------

#[bench]
fn greedy_alternating_256(b: &mut Bencher) {
    bench_greedy(b, &alternating(256));
}

#[bench]
fn greedy_alternating_1024(b: &mut Bencher) {
    bench_greedy(b, &alternating(1024));
}

// --- Realistic few-run inputs (k tiny): DP cost should be ~flat in n ---------

#[bench]
fn dp_numeric_1000(b: &mut Bencher) {
    bench_dp(b, &vec![b'7'; 1000]);
}

#[bench]
fn dp_url_like(b: &mut Bencher) {
    bench_dp(b, b"HTTPS://EXAMPLE.COM/PATH?ID=1234567890&X=ABCDEFGH");
}
