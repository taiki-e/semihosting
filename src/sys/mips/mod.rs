// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Low-level access to MIPS Unified Hosting Interface (UHI).
//!
//! Refs:
//! - Unified Hosting Interface. MD01069 Reference Manual
//! - <https://github.com/qemu/qemu/blob/v7.2.0/target/mips/tcg/sysemu/mips-semi.c>
//! - <https://github.com/qemu/qemu/blob/v10.2.0/target/mips/tcg/system/mips-semi.c>

#![allow(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)] // TODO

pub(crate) mod errno;
#[cfg(feature = "fs")]
pub(crate) mod fs;
pub mod syscall;

use core::{
    ffi::{self, CStr},
    mem,
};

use self::syscall::{
    OperationCode, ParamRegR, ParamRegW, RetReg, syscall0, syscall1_readonly, syscall2,
    syscall2_readonly, syscall3, syscall3_readonly, syscall4, syscall4_readonly,
};
use crate::{
    fd::{BorrowedFd, OwnedFd, RawFd},
    io::{Error, Result},
};

#[allow(missing_docs)]
mod consts {
    pub const O_RDONLY: i32 = 0x0;
    pub const O_WRONLY: i32 = 0x1;
    pub const O_RDWR: i32 = 0x2;
    pub const O_APPEND: i32 = 0x8;
    pub const O_CREAT: i32 = 0x200;
    pub const O_TRUNC: i32 = 0x400;
    pub const O_EXCL: i32 = 0x800;

    pub const S_IXOTH: i32 = 0o1;
    pub const S_IWOTH: i32 = 0o2;
    pub const S_IROTH: i32 = 0o4;
    pub const S_IRWXO: i32 = 0o7;
    pub const S_IXGRP: i32 = 0o10;
    pub const S_IWGRP: i32 = 0o20;
    pub const S_IRGRP: i32 = 0o40;
    pub const S_IRWXG: i32 = 0o70;
    pub const S_IXUSR: i32 = 0o100;
    pub const S_IWUSR: i32 = 0o200;
    pub const S_IRUSR: i32 = 0o400;
    pub const S_IRWXU: i32 = 0o700;
}
pub use self::consts::*;

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
#[repr(i32)]
#[non_exhaustive]
pub enum SeekWhence {
    // Unified Hosting Interface. MD01069 Reference Manual says SEEK_* are defined as follows,
    // but QEMU (as of 7.2) uses Linux's SEEK_* number.
    // SEEK_SET = 0x0001,
    // SEEK_CUR = 0x0002,
    // SEEK_END = 0x0004,
    SEEK_SET = 0,
    SEEK_CUR = 1,
    SEEK_END = 2,
}

#[allow(missing_docs)]
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct uhi_stat {
    pub st_dev: i16,
    pub st_ino: u16,
    pub st_mode: u32,
    pub st_nlink: u16,
    pub st_uid: u16,
    pub st_gid: u16,
    pub st_rdev: i16,
    pub st_size: u64,
    pub st_atime: u64,
    pub st_spare1: u64,
    pub st_mtime: u64,
    pub st_spare2: u64,
    pub st_ctime: u64,
    pub st_spare3: u64,
    pub st_blksize: u64,
    pub st_blocks: u64,
    pub st_spare4: [u64; 2],
}

pub(crate) fn from_errno(res: RetReg) -> Error {
    Error::from_raw_os_error(res.errno())
}

/// UHI_EXIT
#[allow(clippy::cast_sign_loss)]
pub fn mips_exit(code: i32) {
    unsafe {
        syscall1_readonly(OperationCode::UHI_EXIT, ParamRegR::usize(code as isize as usize));
    }
}
pub(crate) use self::mips_exit as exit;

/// UHI_OPEN
#[allow(clippy::cast_sign_loss)]
pub fn mips_open(path: &CStr, flags: i32, mode: i32) -> Result<OwnedFd> {
    let (res, errno) = unsafe {
        syscall3_readonly(
            OperationCode::UHI_OPEN,
            ParamRegR::c_str(path),
            ParamRegR::usize(flags as usize),
            ParamRegR::usize(mode as usize),
        )
    };
    match res.raw_fd() {
        Some(fd) => Ok(unsafe { OwnedFd::from_raw_fd(fd) }),
        None => Err(from_errno(errno)),
    }
}
#[cfg(feature = "stdio")]
const STDIN_FILENO: RawFd = 0; // /dev/stdin
#[cfg(feature = "stdio")]
const STDOUT_FILENO: RawFd = 1; // /dev/stdout
const STDERR_FILENO: RawFd = 2; // /dev/stderr
#[cfg(feature = "stdio")]
pub(crate) type StdioFd = BorrowedFd<'static>;
#[cfg(feature = "stdio")]
pub(crate) fn stdin() -> Result<StdioFd> {
    Ok(unsafe { BorrowedFd::borrow_raw(STDIN_FILENO) })
}
#[cfg(feature = "stdio")]
pub(crate) fn stdout() -> Result<StdioFd> {
    Ok(unsafe { BorrowedFd::borrow_raw(STDOUT_FILENO) })
}
#[cfg(feature = "stdio")]
pub(crate) fn stderr() -> Result<StdioFd> {
    Ok(unsafe { BorrowedFd::borrow_raw(STDERR_FILENO) })
}
#[inline]
#[allow(clippy::cast_sign_loss)]
pub(crate) fn should_close(fd: &OwnedFd) -> bool {
    // In UHI, stdio streams are open by default, and shouldn't closed.
    fd.as_raw_fd() as ffi::c_uint > STDERR_FILENO as ffi::c_uint
}

/// UHI_CLOSE
pub unsafe fn mips_close(fd: RawFd) -> Result<()> {
    let (res, errno) =
        unsafe { syscall1_readonly(OperationCode::UHI_CLOSE, ParamRegR::raw_fd(fd)) };
    if res.usize() == 0 {
        Ok(())
    } else {
        debug_assert_eq!(res.int(), -1);
        Err(from_errno(errno))
    }
}
pub(crate) use self::mips_close as close;

/// UHI_READ
pub fn mips_read(fd: BorrowedFd<'_>, buf: &mut [u8]) -> Result<usize> {
    let len = buf.len();
    let (res, errno) = unsafe {
        syscall3(
            OperationCode::UHI_READ,
            ParamRegW::fd(fd),
            ParamRegW::buf(buf),
            ParamRegW::usize(len),
        )
    };
    if res.int() == -1 {
        Err(from_errno(errno))
    } else {
        debug_assert!(res.usize() <= buf.len());
        Ok(res.usize())
    }
}
#[cfg(any(feature = "stdio", feature = "fs"))]
pub(crate) use self::mips_read as read;

/// UHI_WRITE
pub fn mips_write(fd: BorrowedFd<'_>, buf: &[u8]) -> Result<usize> {
    let (res, errno) = unsafe {
        syscall3_readonly(
            OperationCode::UHI_WRITE,
            ParamRegR::fd(fd),
            ParamRegR::buf(buf),
            ParamRegR::usize(buf.len()),
        )
    };
    if res.int() == -1 {
        Err(from_errno(errno))
    } else {
        debug_assert!(res.usize() <= buf.len());
        Ok(res.usize())
    }
}
#[cfg(any(feature = "stdio", feature = "fs"))]
pub(crate) use self::mips_write as write;

/// UHI_LSEEK
pub unsafe fn mips_lseek(fd: BorrowedFd<'_>, offset: isize, whence: SeekWhence) -> Result<usize> {
    let (res, errno) = unsafe {
        syscall3_readonly(
            OperationCode::UHI_LSEEK,
            ParamRegR::fd(fd),
            ParamRegR::isize(offset),
            ParamRegR::usize(whence as usize),
        )
    };
    if res.int() == -1 { Err(from_errno(errno)) } else { Ok(res.usize()) }
}

/// UHI_UNLINK
pub fn mips_unlink(path: &CStr) -> Result<()> {
    let (res, errno) =
        unsafe { syscall1_readonly(OperationCode::UHI_UNLINK, ParamRegR::c_str(path)) };
    if res.usize() == 0 { Ok(()) } else { Err(from_errno(errno)) }
}

/// UHI_FSTAT
pub fn mips_fstat(fd: BorrowedFd<'_>) -> Result<uhi_stat> {
    let mut buf: uhi_stat = unsafe { mem::zeroed() };
    let (res, errno) =
        unsafe { syscall2(OperationCode::UHI_FSTAT, ParamRegW::fd(fd), ParamRegW::ref_(&mut buf)) };
    if res.usize() == 0 { Ok(buf) } else { Err(from_errno(errno)) }
}
#[cfg(feature = "stdio")]
pub(crate) fn is_terminal(fd: BorrowedFd<'_>) -> bool {
    const S_IFCHR: u32 = 0x2000;
    matches!(mips_fstat(fd), Ok(stat) if stat.st_mode & S_IFCHR != 0)
}

/// UHI_ARGC
pub fn mips_argc() -> usize {
    let (res, _errno) = unsafe { syscall0(OperationCode::UHI_ARGC) };
    debug_assert!(!res.int().is_negative(), "{}", res.int());
    res.usize()
}

/// UHI_ARGNLEN
pub fn mips_argnlen(n: usize) -> Result<usize> {
    let (res, errno) =
        unsafe { syscall1_readonly(OperationCode::UHI_ARGNLEN, ParamRegR::usize(n)) };
    if res.int() == -1 { Err(from_errno(errno)) } else { Ok(res.usize()) }
}

/// UHI_ARGN
pub unsafe fn mips_argn(n: usize, buf: *mut u8) -> Result<()> {
    let (res, errno) =
        unsafe { syscall2(OperationCode::UHI_ARGN, ParamRegW::usize(n), ParamRegW::ptr(buf)) };
    if res.usize() == 0 {
        Ok(())
    } else {
        debug_assert_eq!(res.int(), -1);
        Err(from_errno(errno))
    }
}

/// UHI_PLOG
pub fn mips_plog(msg: &CStr) -> Result<usize> {
    let (res, errno) = unsafe { syscall1_readonly(OperationCode::UHI_PLOG, ParamRegR::c_str(msg)) };
    if res.int() == -1 {
        Err(from_errno(errno))
    } else {
        debug_assert_eq!(res.usize(), msg.to_bytes().len());
        Ok(res.usize())
    }
}

// TODO: UHI_ASSERT

/// UHI_PREAD
pub fn mips_pread(fd: BorrowedFd<'_>, buf: &mut [u8], offset: usize) -> Result<usize> {
    let len = buf.len();
    let (res, errno) = unsafe {
        syscall4(
            OperationCode::UHI_PREAD,
            ParamRegW::fd(fd),
            ParamRegW::buf(buf),
            ParamRegW::usize(len),
            ParamRegW::usize(offset),
        )
    };
    if res.int() == -1 {
        Err(from_errno(errno))
    } else {
        debug_assert!(res.usize() <= buf.len());
        Ok(res.usize())
    }
}

/// UHI_PWRITE
pub fn mips_pwrite(fd: BorrowedFd<'_>, buf: &[u8], offset: usize) -> Result<usize> {
    let (res, errno) = unsafe {
        syscall4_readonly(
            OperationCode::UHI_PWRITE,
            ParamRegR::fd(fd),
            ParamRegR::buf(buf),
            ParamRegR::usize(buf.len()),
            ParamRegR::usize(offset),
        )
    };
    if res.int() == -1 {
        Err(from_errno(errno))
    } else {
        debug_assert!(res.usize() <= buf.len());
        Ok(res.usize())
    }
}

/// UHI_LINK
pub fn mips_link(old: &CStr, new: &CStr) -> Result<()> {
    let (res, errno) = unsafe {
        syscall2_readonly(OperationCode::UHI_LINK, ParamRegR::c_str(old), ParamRegR::c_str(new))
    };
    if res.usize() == 0 {
        Ok(())
    } else {
        debug_assert_eq!(res.int(), -1);
        Err(from_errno(errno))
    }
}
