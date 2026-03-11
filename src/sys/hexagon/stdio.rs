// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::{OpenMode, sys_istty, sys_open};
use crate::{
    fd::{BorrowedFd, OwnedFd},
    io,
};

pub(crate) type StdioFd = OwnedFd;

pub(crate) fn stdin() -> io::Result<StdioFd> {
    sys_open(c!("/dev/stdin"), OpenMode::RDONLY)
}
pub(crate) fn stdout() -> io::Result<StdioFd> {
    sys_open(c!("/dev/stdout"), OpenMode::WRONLY_TRUNC)
}
pub(crate) fn stderr() -> io::Result<StdioFd> {
    // if failed, redirect to stdout
    sys_open(c!("/dev/stderr"), OpenMode::WRONLY_APPEND).or_else(|_| stdout())
}

#[inline]
pub(crate) fn should_close(_fd: &OwnedFd) -> bool {
    // In Hexagon semihosting, stdio streams are handled like normal fd.
    true
}

pub(crate) fn is_terminal(fd: BorrowedFd<'_>) -> bool {
    sys_istty(fd).unwrap_or(false)
}
