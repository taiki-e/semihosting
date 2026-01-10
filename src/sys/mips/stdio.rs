// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, mips_fstat};
use crate::{fd::BorrowedFd, io};

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

pub(crate) fn is_terminal(fd: BorrowedFd<'_>) -> bool {
    const S_IFCHR: u32 = 0x2000;
    matches!(mips_fstat(fd), Ok(stat) if stat.st_mode & S_IFCHR != 0)
}
