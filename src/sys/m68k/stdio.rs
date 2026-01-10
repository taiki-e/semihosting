// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::{O_APPEND, O_RDONLY, O_WRONLY, hosted_isatty, hosted_open};
use crate::{
    fd::{BorrowedFd, OwnedFd},
    io,
};

pub(crate) type StdioFd = OwnedFd;
pub(crate) fn stdin() -> io::Result<StdioFd> {
    hosted_open(c!("/dev/stdin"), O_RDONLY, 0o666)
}
pub(crate) fn stdout() -> io::Result<StdioFd> {
    hosted_open(c!("/dev/stdout"), O_WRONLY | O_APPEND, 0o666)
}
pub(crate) fn stderr() -> io::Result<StdioFd> {
    hosted_open(c!("/dev/stderr"), O_WRONLY | O_APPEND, 0o666)
}

pub(crate) fn is_terminal(fd: BorrowedFd<'_>) -> bool {
    hosted_isatty(fd).unwrap_or(false)
}
