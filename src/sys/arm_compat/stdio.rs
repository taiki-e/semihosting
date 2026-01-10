// SPDX-License-Identifier: Apache-2.0 OR MIT

use super::{OpenMode, sys_istty, sys_open};
use crate::{
    fd::{BorrowedFd, OwnedFd},
    io,
};

// From https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-open-0x01:
// > ARM targets interpret the special path name `:tt` as meaning the console
// > input stream, for an open-read or the console output stream, for an open-write.
// > Opening these streams is performed as part of the standard startup code for
// > those applications that reference the C `stdio` streams.
// And, if the SH_EXT_STDOUT_STDERR semihosting extension is supported:
// > If the special path name `:tt` is opened with an `fopen` mode requesting write access (`w`, `wb`, `w+`, or `w+b`), then this is a request to open `stdout`.
// > If the special path name `:tt` is opened with a mode requesting append access (`a`, `ab`, `a+`, or `a+b`), then this is a request to open `stderr`.
pub(crate) type StdioFd = OwnedFd;
pub(crate) fn stdin() -> io::Result<StdioFd> {
    sys_open(c!(":tt"), OpenMode::RDONLY)
}
pub(crate) fn stdout() -> io::Result<StdioFd> {
    sys_open(c!(":tt"), OpenMode::WRONLY_TRUNC)
}
pub(crate) fn stderr() -> io::Result<StdioFd> {
    // if failed, redirect to stdout
    sys_open(c!(":tt"), OpenMode::WRONLY_APPEND).or_else(|_| stdout())
}

pub(crate) fn is_terminal(fd: BorrowedFd<'_>) -> bool {
    sys_istty(fd).unwrap_or(false)
}
