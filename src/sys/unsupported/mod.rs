// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Stub implementations for unsupported host targets.
//!
//! All functions panic at runtime. This module exists only so that the crate
//! compiles on non-semihosting targets (e.g., x86_64) for use in tests,
//! proc-macro crates, or build scripts that depend on types from this crate.

pub(crate) mod errno;
#[cfg(feature = "fs")]
pub(crate) mod fs;
#[cfg(feature = "stdio")]
pub(crate) mod stdio;

use core::mem::MaybeUninit;

use crate::{
    fd::{BorrowedFd, RawFd},
    io,
};

pub(crate) fn exit(_code: i32) -> ! {
    unimplemented!("semihosting is not available on this target")
}

pub(crate) unsafe fn close(_fd: RawFd) -> io::Result<()> {
    unimplemented!("semihosting is not available on this target")
}

pub(crate) fn read(_fd: BorrowedFd<'_>, _buf: &mut [u8]) -> io::Result<usize> {
    unimplemented!("semihosting is not available on this target")
}

pub(crate) fn read_uninit<'a>(
    _fd: BorrowedFd<'_>,
    _buf: &'a mut [MaybeUninit<u8>],
) -> io::Result<(&'a mut [u8], &'a mut [MaybeUninit<u8>])> {
    unimplemented!("semihosting is not available on this target")
}

pub(crate) fn write(_fd: BorrowedFd<'_>, _buf: &[u8]) -> io::Result<usize> {
    unimplemented!("semihosting is not available on this target")
}
