// SPDX-FileCopyrightText: 2014 kennytm
// SPDX-FileCopyrightText: 2024 Michael Spiegel
// SPDX-FileCopyrightText: 2025 Shun Sakai
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Interleaving of codeword blocks.

use alloc::vec::Vec;
use core::ops::Deref;

/// This method interleaves a vector of slices into a single vector.
///
/// It will first insert all the first elements of the slices in `blocks`, then
/// all the second elements, then all the third elements, and so on.
///
/// The longest slice must be at the last of `blocks`, and `blocks` must not be
/// empty.
pub fn interleave<T: Copy, V: Deref<Target = [T]>>(blocks: &[V]) -> Vec<T> {
    let last_block_len = blocks.last().unwrap().len();
    let mut res = Vec::with_capacity(last_block_len * blocks.len());
    for i in 0..last_block_len {
        for t in blocks {
            if i < t.len() {
                res.push(t[i]);
            }
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interleave() {
        let res = interleave(&[&b"1234"[..], b"5678", b"abcdef", b"ghijkl"]);
        assert_eq!(res, b"15ag26bh37ci48djekfl");
    }
}
