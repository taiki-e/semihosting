// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{fd::BorrowedFd, io};

pub(crate) type StdioFd = BorrowedFd<'static>;
pub(crate) fn stdin() -> io::Result<StdioFd> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
pub(crate) fn stdout() -> io::Result<StdioFd> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
pub(crate) fn stderr() -> io::Result<StdioFd> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}

pub(crate) fn is_terminal(fd: BorrowedFd<'_>) -> bool {
    // TODO
    false
}
