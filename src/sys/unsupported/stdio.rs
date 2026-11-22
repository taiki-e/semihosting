// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
    fd::{BorrowedFd, OwnedFd},
    io,
};

pub(crate) type StdioFd = OwnedFd;

pub(crate) fn stdin() -> io::Result<StdioFd> {
    unimplemented!("semihosting is not available on this target")
}
pub(crate) fn stdout() -> io::Result<StdioFd> {
    unimplemented!("semihosting is not available on this target")
}
pub(crate) fn stderr() -> io::Result<StdioFd> {
    unimplemented!("semihosting is not available on this target")
}

#[inline]
pub(crate) fn should_close(_fd: &OwnedFd) -> bool {
    true
}

pub(crate) fn is_terminal(_fd: BorrowedFd<'_>) -> bool {
    false
}
