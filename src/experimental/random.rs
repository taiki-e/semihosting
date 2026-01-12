// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Random value generation.

use core::mem::MaybeUninit;

use crate::{io, sys::random as sys};

/// Fills `bytes` with random bytes.
///
/// # Underlying sources
///
/// This is currently implemented by reading the host system's `/dev/urandom`.
///
/// On the host system does not have `/dev/urandom` like Windows, this always returns an error.
///
/// Note that the sources used might change over time.
#[inline]
pub fn fill_bytes(bytes: &mut [u8]) -> io::Result<()> {
    let len = bytes.len();
    // SAFETY: transmuting initialized `&mut [u8]` to `&mut [MaybeUninit<u8>]` is safe unless uninitialized byte will be written to resulting slice.
    let bytes = unsafe {
        core::slice::from_raw_parts_mut(bytes.as_mut_ptr().cast::<MaybeUninit<u8>>(), len)
    };
    fill_uninit_bytes(bytes)?;
    Ok(())
}

/// Fills `bytes` with random bytes.
///
/// Unlike [`fill_bytes`], this takes potentially uninitialized bytes.
///
/// See [`fill_bytes`] for details.
#[inline]
pub fn fill_uninit_bytes(bytes: &mut [MaybeUninit<u8>]) -> io::Result<&mut [u8]> {
    sys::fill_bytes(bytes)
}
