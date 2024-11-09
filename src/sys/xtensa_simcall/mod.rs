// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Low-level access to Xtensa Tensilica ISS SIMCALL.
//!
//! Refs:
//! - <https://github.com/qemu/qemu/blob/v9.1.0/target/xtensa/xtensa-semi.c>

#![allow(clippy::missing_safety_doc)] // TODO
#![allow(unused_variables)] // TODO

#[cfg(feature = "fs")]
pub(crate) mod fs {
    use core::ffi::CStr;

    use crate::{
        fd::{BorrowedFd, OwnedFd},
        io::{self, Error, Result},
    };

    // TODO
    pub(crate) struct Metadata {}
    impl Metadata {
        pub(crate) fn size(&self) -> u64 {
            0
        }
    }
    pub(crate) fn metadata(fd: BorrowedFd<'_>) -> Result<Metadata> {
        // TODO
        Err(io::ErrorKind::Unsupported.into())
    }
    pub(crate) fn open(path: &CStr, options: &crate::fs::OpenOptions) -> Result<OwnedFd> {
        // TODO
        Err(io::ErrorKind::Unsupported.into())
    }
    pub(crate) fn seek(fd: BorrowedFd<'_>, pos: io::SeekFrom) -> Result<u64> {
        // TODO
        Err(io::ErrorKind::Unsupported.into())
    }
    pub(crate) fn unlink(path: &CStr) -> Result<()> {
        // TODO
        Err(io::ErrorKind::Unsupported.into())
    }
    pub(crate) fn rename(_from: &CStr, _to: &CStr) -> Result<()> {
        // TODO
        Err(io::ErrorKind::Unsupported.into())
    }
}

const SYS_exit: i32 = 1;
const SYS_read: i32 = 3;
const SYS_write: i32 = 4;
const SYS_open: i32 = 5;
const SYS_close: i32 = 6;
const SYS_lseek: i32 = 19;

use crate::{
    fd::{BorrowedFd, OwnedFd, RawFd},
    io::{self, Error, Result},
};

// #[link(name = "semi")]
// extern "C" {
//     fn _semihosting_syscall(a: i32, b: i32, c: i32, d: i32) -> i32;
// }

pub(crate) fn exit(code: i32) {
    // error: unrecognized instruction mnemonic
    unsafe {
        use core::arch::asm;

        asm!(
            "simcall",
            inout("a2") SYS_exit => _,
            inout("a3") code => _,
            in("a4") 0_usize,
            in("a5") 0_usize,
            options(nostack),
        )
    }

    // unsafe {
    //     _semihosting_syscall(SYS_exit, code, 0, 0);
    // }
}

#[cfg(any(feature = "stdio", feature = "fs"))]
pub(crate) fn read(fd: BorrowedFd<'_>, buf: &mut [u8]) -> Result<usize> {
    // // TODO: correct args?
    // let res = unsafe { _semihosting_syscall(SYS_read, fd.as_raw_fd(), 0, 0) };
    // if res < 0 {
    //     // TODO: errno
    //     Err(io::ErrorKind::Unsupported.into())
    // } else {
    //     Ok(res as usize)
    // }
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
#[cfg(any(feature = "stdio", feature = "fs"))]
pub(crate) fn write(fd: BorrowedFd<'_>, buf: &[u8]) -> Result<usize> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
#[cfg(feature = "stdio")]
pub(crate) fn is_terminal(fd: BorrowedFd<'_>) -> bool {
    // TODO
    false
}
#[cfg(feature = "stdio")]
pub(crate) type StdioFd = BorrowedFd<'static>;
#[cfg(feature = "stdio")]
pub(crate) fn stdin() -> Result<StdioFd> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
#[cfg(feature = "stdio")]
pub(crate) fn stdout() -> Result<StdioFd> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
#[cfg(feature = "stdio")]
pub(crate) fn stderr() -> Result<StdioFd> {
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
