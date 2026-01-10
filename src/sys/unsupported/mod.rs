// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(missing_docs, unused_variables)] // TODO
#![allow(clippy::missing_safety_doc)] // TODO

#[cfg(feature = "fs")]
pub(crate) mod fs;

use crate::{
    fd::{BorrowedFd, OwnedFd, RawFd},
    io::{self, Error, Result},
};

pub(crate) fn exit(code: i32) {
    // TODO
}

#[cfg(any(feature = "stdio", feature = "fs"))]
pub(crate) fn read(fd: BorrowedFd<'_>, buf: &mut [u8]) -> Result<usize> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
#[cfg(any(feature = "stdio", feature = "fs"))]
pub(crate) fn write(fd: BorrowedFd<'_>, buf: &[u8]) -> Result<usize> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
#[inline]
pub(crate) fn should_close(fd: &OwnedFd) -> bool {
    // TODO
    true
}
pub unsafe fn close(fd: RawFd) -> Result<()> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
