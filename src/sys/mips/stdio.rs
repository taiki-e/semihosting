// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::ffi::c_uint;

use super::mips_fstat;
use crate::{
    fd::{BorrowedFd, OwnedFd, RawFd},
    io,
};

const STDIN_FILENO: RawFd = 0; // /dev/stdin
const STDOUT_FILENO: RawFd = 1; // /dev/stdout
const STDERR_FILENO: RawFd = 2; // /dev/stderr

pub(crate) type StdioFd = BorrowedFd<'static>;

pub(crate) fn stdin() -> io::Result<StdioFd> {
    Ok(unsafe { BorrowedFd::borrow_raw(STDIN_FILENO) })
}
pub(crate) fn stdout() -> io::Result<StdioFd> {
    Ok(unsafe { BorrowedFd::borrow_raw(STDOUT_FILENO) })
}
pub(crate) fn stderr() -> io::Result<StdioFd> {
    Ok(unsafe { BorrowedFd::borrow_raw(STDERR_FILENO) })
}

#[inline]
#[allow(clippy::cast_sign_loss)]
pub(crate) fn should_close(fd: &OwnedFd) -> bool {
    // In UHI, stdio streams are open by default, and shouldn't closed.
    fd.as_raw_fd() as c_uint > STDERR_FILENO as c_uint
}

pub(crate) fn is_terminal(fd: BorrowedFd<'_>) -> bool {
    const S_IFCHR: u32 = 0x2000;
    matches!(mips_fstat(fd), Ok(stat) if stat.st_mode & S_IFCHR != 0)
}
