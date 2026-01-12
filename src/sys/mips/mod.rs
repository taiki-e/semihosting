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
#[cfg(feature = "stdio")]
pub(crate) mod stdio;
pub mod syscall;

use core::{ffi::CStr, mem, mem::MaybeUninit};

use self::syscall::{
    OperationCode, ParamRegR, ParamRegW, RetReg, syscall0, syscall1_noreturn_readonly,
    syscall1_readonly, syscall2, syscall2_readonly, syscall3, syscall3_readonly, syscall4,
    syscall4_readonly,
};
use crate::{
    fd::{BorrowedFd, OwnedFd, RawFd},
    io,
    utils::slice_assume_init_mut,
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
#[allow(clippy::exhaustive_structs)] // TODO(semver)
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

#[cold]
fn from_errno(res: RetReg) -> io::Error {
    io::Error::from_raw_os_error(res.errno())
}

/// UHI_exit
// TODO(semver): change return type to !?
pub fn mips_exit(code: i32) {
    #[allow(clippy::cast_sign_loss)]
    unsafe {
        syscall1_readonly(OperationCode::UHI_EXIT, ParamRegR::unsigned(code as isize as usize));
    }
}
pub(crate) fn exit(code: i32) -> ! {
    #[allow(clippy::cast_sign_loss)]
    unsafe {
        syscall1_noreturn_readonly(
            OperationCode::UHI_EXIT,
            ParamRegR::unsigned(code as isize as usize),
        )
    }
}

/// UHI_open
pub fn mips_open(path: &CStr, flags: i32, mode: i32) -> io::Result<OwnedFd> {
    #[allow(clippy::cast_sign_loss)]
    let (res, errno) = unsafe {
        syscall3_readonly(
            OperationCode::UHI_OPEN,
            ParamRegR::c_str(path),
            ParamRegR::unsigned(flags as usize),
            ParamRegR::unsigned(mode as usize),
        )
    };
    match res.raw_fd() {
        Some(fd) => Ok(unsafe { OwnedFd::from_raw_fd(fd) }),
        None => Err(from_errno(errno)),
    }
}

/// UHI_close
/// (Equivalent to [`sys::close`](crate::sys::close))
pub unsafe fn mips_close(fd: RawFd) -> io::Result<()> {
    let (res, errno) =
        unsafe { syscall1_readonly(OperationCode::UHI_CLOSE, ParamRegR::raw_fd(fd)) };
    if res.unsigned() == 0 {
        Ok(())
    } else {
        debug_assert_eq!(res.signed(), -1);
        Err(from_errno(errno))
    }
}
pub(crate) use self::mips_close as close;

/// UHI_read
/// (Equivalent to [`sys::read`](crate::sys::read))
pub fn mips_read(fd: BorrowedFd<'_>, buf: &mut [u8]) -> io::Result<usize> {
    let len = buf.len();
    // SAFETY: transmuting initialized `&mut [u8]` to `&mut [MaybeUninit<u8>]` is safe unless uninitialized byte will be written to resulting slice.
    let buf =
        unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr().cast::<MaybeUninit<u8>>(), len) };
    Ok(read_uninit(fd, buf)?.0.len())
}
pub(crate) use self::mips_read as read;
pub(crate) fn read_uninit<'a>(
    fd: BorrowedFd<'_>,
    buf: &'a mut [MaybeUninit<u8>],
) -> io::Result<(&'a mut [u8], &'a mut [MaybeUninit<u8>])> {
    let len = buf.len();
    let (res, errno) = unsafe {
        syscall3(
            OperationCode::UHI_READ,
            ParamRegW::fd(fd),
            ParamRegW::buf(buf),
            ParamRegW::unsigned(len),
        )
    };
    if res.signed() == -1 {
        Err(from_errno(errno))
    } else {
        debug_assert!(res.unsigned() <= buf.len());
        let (filled, rest) = buf.split_at_mut(res.unsigned());
        Ok((unsafe { slice_assume_init_mut(filled) }, rest))
    }
}

/// UHI_write
/// (Equivalent to [`sys::write`](crate::sys::write))
pub fn mips_write(fd: BorrowedFd<'_>, buf: &[u8]) -> io::Result<usize> {
    let (res, errno) = unsafe {
        syscall3_readonly(
            OperationCode::UHI_WRITE,
            ParamRegR::fd(fd),
            ParamRegR::buf(buf),
            ParamRegR::unsigned(buf.len()),
        )
    };
    if res.signed() == -1 {
        Err(from_errno(errno))
    } else {
        debug_assert!(res.unsigned() <= buf.len());
        Ok(res.unsigned())
    }
}
pub(crate) use self::mips_write as write;

/// UHI_lseek
pub unsafe fn mips_lseek(
    fd: BorrowedFd<'_>,
    offset: isize,
    whence: SeekWhence,
) -> io::Result<usize> {
    let (res, errno) = unsafe {
        syscall3_readonly(
            OperationCode::UHI_LSEEK,
            ParamRegR::fd(fd),
            ParamRegR::signed(offset),
            ParamRegR::unsigned(whence as usize),
        )
    };
    if res.signed() == -1 { Err(from_errno(errno)) } else { Ok(res.unsigned()) }
}

/// UHI_unlink
/// (Equivalent to [`fs::remove_file`](crate::fs::remove_file))
pub fn mips_unlink(path: &CStr) -> io::Result<()> {
    let (res, errno) =
        unsafe { syscall1_readonly(OperationCode::UHI_UNLINK, ParamRegR::c_str(path)) };
    if res.unsigned() == 0 { Ok(()) } else { Err(from_errno(errno)) }
}

/// UHI_fstat
pub fn mips_fstat(fd: BorrowedFd<'_>) -> io::Result<uhi_stat> {
    let mut buf: uhi_stat = unsafe { mem::zeroed() };
    let (res, errno) =
        unsafe { syscall2(OperationCode::UHI_FSTAT, ParamRegW::fd(fd), ParamRegW::ref_(&mut buf)) };
    if res.unsigned() == 0 { Ok(buf) } else { Err(from_errno(errno)) }
}

/// UHI_argc
pub fn mips_argc() -> usize {
    let (res, _errno) = unsafe { syscall0(OperationCode::UHI_ARGC) };
    debug_assert!(!res.signed().is_negative(), "{}", res.signed());
    res.unsigned()
}

/// UHI_argnlen
pub fn mips_argnlen(n: usize) -> io::Result<usize> {
    let (res, errno) =
        unsafe { syscall1_readonly(OperationCode::UHI_ARGNLEN, ParamRegR::unsigned(n)) };
    if res.signed() == -1 { Err(from_errno(errno)) } else { Ok(res.unsigned()) }
}

/// UHI_argn
pub unsafe fn mips_argn(n: usize, buf: *mut u8) -> io::Result<()> {
    let (res, errno) =
        unsafe { syscall2(OperationCode::UHI_ARGN, ParamRegW::unsigned(n), ParamRegW::ptr(buf)) };
    if res.unsigned() == 0 {
        Ok(())
    } else {
        debug_assert_eq!(res.signed(), -1);
        Err(from_errno(errno))
    }
}

/// UHI_plog
pub fn mips_plog(msg: &CStr) -> io::Result<usize> {
    let (res, errno) = unsafe { syscall1_readonly(OperationCode::UHI_PLOG, ParamRegR::c_str(msg)) };
    if res.signed() == -1 {
        Err(from_errno(errno))
    } else {
        debug_assert_eq!(res.unsigned(), msg.to_bytes().len());
        Ok(res.unsigned())
    }
}

// TODO(mips): UHI_ASSERT

/// UHI_pread
pub fn mips_pread(fd: BorrowedFd<'_>, buf: &mut [u8], offset: usize) -> io::Result<usize> {
    let len = buf.len();
    // SAFETY: transmuting initialized `&mut [u8]` to `&mut [MaybeUninit<u8>]` is safe unless uninitialized byte will be written to resulting slice.
    let buf =
        unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr().cast::<MaybeUninit<u8>>(), len) };
    Ok(mips_pread_uninit(fd, buf, offset)?.0.len())
}
fn mips_pread_uninit<'a>(
    fd: BorrowedFd<'_>,
    buf: &'a mut [MaybeUninit<u8>],
    offset: usize,
) -> io::Result<(&'a mut [u8], &'a mut [MaybeUninit<u8>])> {
    let len = buf.len();
    let (res, errno) = unsafe {
        syscall4(
            OperationCode::UHI_PREAD,
            ParamRegW::fd(fd),
            ParamRegW::buf(buf),
            ParamRegW::unsigned(len),
            ParamRegW::unsigned(offset),
        )
    };
    if res.signed() == -1 {
        Err(from_errno(errno))
    } else {
        debug_assert!(res.unsigned() <= buf.len());
        let (filled, rest) = buf.split_at_mut(res.unsigned());
        Ok((unsafe { slice_assume_init_mut(filled) }, rest))
    }
}

/// UHI_pwrite
pub fn mips_pwrite(fd: BorrowedFd<'_>, buf: &[u8], offset: usize) -> io::Result<usize> {
    let (res, errno) = unsafe {
        syscall4_readonly(
            OperationCode::UHI_PWRITE,
            ParamRegR::fd(fd),
            ParamRegR::buf(buf),
            ParamRegR::unsigned(buf.len()),
            ParamRegR::unsigned(offset),
        )
    };
    if res.signed() == -1 {
        Err(from_errno(errno))
    } else {
        debug_assert!(res.unsigned() <= buf.len());
        Ok(res.unsigned())
    }
}

/// UHI_link
pub fn mips_link(old: &CStr, new: &CStr) -> io::Result<()> {
    let (res, errno) = unsafe {
        syscall2_readonly(OperationCode::UHI_LINK, ParamRegR::c_str(old), ParamRegR::c_str(new))
    };
    if res.unsigned() == 0 {
        Ok(())
    } else {
        debug_assert_eq!(res.signed(), -1);
        Err(from_errno(errno))
    }
}
