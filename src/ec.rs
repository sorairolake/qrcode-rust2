// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The `ec` module applies the Reed-Solomon error correction codes.

mod capacity;
mod construct_codewords;
mod error_correction_sizes;
mod gf256;
mod interleave;

use alloc::vec::Vec;

use self::gf256::{EXP_TABLE, GENERATOR_POLYNOMIALS, LOG_TABLE};
pub use self::{capacity::max_allowed_errors, construct_codewords::construct_codewords};

/// Creates the error correction code in N bytes.
///
/// This method only supports computing the error-correction code up to 69
/// bytes. Longer blocks will result in task panic.
///
/// This method treats the data as a polynomial of the form (a\[0\]
/// x<sup>m+n</sup> + a\[1\] x<sup>m+n-1</sup> + … + a\[m\] x<sup>n</sup>) in
/// GF(2<sup>8</sup>), and then computes the polynomial modulus with a generator
/// polynomial of degree N.
#[must_use]
pub fn create_error_correction_code(data: &[u8], ec_code_size: usize) -> Vec<u8> {
    let data_len = data.len();
    let log_den = GENERATOR_POLYNOMIALS[ec_code_size];

    let mut res = data.to_vec();
    res.resize(ec_code_size + data_len, 0);

    for i in 0..data_len {
        let lead_coeff = res[i] as usize;
        if lead_coeff == 0 {
            continue;
        }

        let log_lead_coeff = usize::from(LOG_TABLE[lead_coeff]);
        for (u, v) in res[i + 1..].iter_mut().zip(log_den.iter()) {
            *u ^= EXP_TABLE[(usize::from(*v) + log_lead_coeff) % 255];
        }
    }

    res.split_off(data_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poly_mod_1() {
        let res = create_error_correction_code(b" [\x0Bx\xD1r\xDCMC@\xEC\x11\xEC\x11\xEC\x11", 10);
        assert_eq!(res, b"\xC4#'w\xEB\xD7\xE7\xE2]\x17");
    }

    #[test]
    fn test_poly_mod_2() {
        let res = create_error_correction_code(b" [\x0Bx\xD1r\xDCMC@\xEC\x11\xEC", 13);
        assert_eq!(res, b"\xA8H\x16R\xD96\x9C\x00.\x0F\xB4z\x10");
    }

    #[test]
    fn test_poly_mod_3() {
        let res = create_error_correction_code(b"CUF\x86W&U\xC2w2\x06\x12\x06g&", 18);
        assert_eq!(res, b"\xD5\xC7\x0B-s\xF7\xF1\xDF\xE5\xF8\x9Au\x9AoV\xA1o'");
    }
}
