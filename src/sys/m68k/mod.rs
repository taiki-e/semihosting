// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Low-level access to m68k Semihosting Protocol.
//!
//! Refs:
//! - <https://sourceware.org/git/?p=newlib-cygwin.git;a=blob;f=libgloss/m68k/m68k-semi.txt;hb=HEAD>
//! - <https://github.com/qemu/qemu/blob/v10.2.0/target/m68k/m68k-semi.c>

// https://github.com/qemu/qemu/blob/v10.2.0/target/m68k/translate.c#L1407
// https://github.com/picolibc/picolibc/blob/056c6ca8d9e79c191695d91534ad852578ca5551/semihost/machine/m68k/m68k_semihost.S#L53

#![allow(clippy::missing_safety_doc)] // TODO
#![allow(unused_variables)] // TODO

pub(crate) mod errno;
#[cfg(feature = "fs")]
pub(crate) mod fs;
#[cfg(feature = "stdio")]
pub(crate) mod stdio;
pub mod syscall;

use core::{
    ffi::{CStr, c_uint},
    mem,
};

use self::syscall::{OperationCode, ParamRegR, ParamRegW, RetReg, syscall, syscall_readonly};
use crate::{
    fd::{BorrowedFd, OwnedFd, RawFd},
    io,
};

#[allow(missing_docs)]
mod gdb {
    // https://sourceware.org/gdb/current/onlinedocs/gdb.html/Integral-Datatypes.html#Integral-Datatypes
    // > int, unsigned int, mode_t and time_t are implemented as 32 bit values in this protocol.
    // pub(super) type int = i32;
    pub(super) type uint = u32;
    pub(super) type mode_t = u32;
    pub(super) type time_t = u32;
    // > long and unsigned long are implemented as 64 bit types.
    pub(super) type long = i64;
    pub(super) type ulong = u64;

    // https://sourceware.org/gdb/current/onlinedocs/gdb.html/Open-Flags.html
    pub const O_RDONLY: u32 = 0x0;
    pub const O_WRONLY: u32 = 0x1;
    pub const O_RDWR: u32 = 0x2;
    pub const O_APPEND: u32 = 0x8;
    pub const O_CREAT: u32 = 0x200;
    pub const O_TRUNC: u32 = 0x400;
    pub const O_EXCL: u32 = 0x800;

    // https://sourceware.org/gdb/current/onlinedocs/gdb.html/mode_005ft-Values.html
    pub const S_IFREG: mode_t = 0o100000;
    pub const S_IFDIR: mode_t = 0o40000;
    pub const S_IRUSR: mode_t = 0o400;
    pub const S_IWUSR: mode_t = 0o200;
    pub const S_IXUSR: mode_t = 0o100;
    pub const S_IRGRP: mode_t = 0o40;
    pub const S_IWGRP: mode_t = 0o20;
    pub const S_IXGRP: mode_t = 0o10;
    pub const S_IROTH: mode_t = 0o4;
    pub const S_IWOTH: mode_t = 0o2;
    pub const S_IXOTH: mode_t = 0o1;

    // https://sourceware.org/gdb/current/onlinedocs/gdb.html/Lseek-Flags.html
    #[allow(missing_docs)]
    #[derive(Debug, Clone, Copy)]
    #[repr(i32)]
    #[non_exhaustive]
    pub enum LseekFlag {
        SEEK_SET = 0,
        SEEK_CUR = 1,
        SEEK_END = 2,
    }

    // https://sourceware.org/gdb/current/onlinedocs/gdb/struct-stat.html
    #[allow(missing_docs)]
    #[allow(clippy::exhaustive_structs)]
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct stat {
        pub st_dev: uint,
        pub st_ino: uint,
        pub st_mode: mode_t,
        pub st_nlink: uint,
        pub st_uid: uint,
        pub st_gid: uint,
        pub st_rdev: uint,
        pub st_size: ulong,
        pub st_blksize: ulong,
        pub st_blocks: ulong,
        pub st_atime: time_t,
        pub st_mtime: time_t,
        pub st_ctime: time_t,
    }

    // https://sourceware.org/gdb/current/onlinedocs/gdb.html/struct-timeval.html#struct-timeval
    #[allow(missing_docs)]
    #[allow(clippy::exhaustive_structs)]
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct timeval {
        pub tv_sec: time_t,
        pub tv_usec: long,
    }
}
pub use self::gdb::*;

pub(crate) fn from_errno(res: RetReg) -> io::Error {
    io::Error::from_raw_os_error(res.errno())
}

/// HOSTED_EXIT
#[allow(clippy::cast_sign_loss)]
pub fn hosted_exit(code: i32) {
    unsafe { syscall_readonly(OperationCode::HOSTED_EXIT, ParamRegR::isize(code as isize)) }
}
pub(crate) use self::hosted_exit as exit;

// TODO: HOSTED_INIT_SIM

/// HOSTED_OPEN
pub fn hosted_open(path: &CStr, flags: u32, mode: mode_t) -> io::Result<OwnedFd> {
    let mut block = [
        ParamRegW::c_str(path),
        ParamRegW::usize(path.to_bytes().len() + 1),
        ParamRegW::usize(flags as usize),
        ParamRegW::usize(mode as usize),
    ];
    unsafe { syscall(OperationCode::HOSTED_OPEN, ParamRegW::block(&mut block)) }
    let res = block[0].to_ret();
    match res.raw_fd() {
        Some(fd) => Ok(unsafe { OwnedFd::from_raw_fd(fd) }),
        None => Err(from_errno(block[1].to_ret())),
    }
}
// #[cfg(feature = "stdio")]
// const STDIN_FILENO: RawFd = 0; // /dev/stdin
// #[cfg(feature = "stdio")]
// const STDOUT_FILENO: RawFd = 1; // /dev/stdout
const STDERR_FILENO: RawFd = 2; // /dev/stderr
#[inline]
#[allow(clippy::cast_sign_loss)]
pub(crate) fn should_close(fd: &OwnedFd) -> bool {
    // TODO
    fd.as_raw_fd() as c_uint > STDERR_FILENO as c_uint
    //    true
}

/// HOSTED_CLOSE
pub unsafe fn hosted_close(fd: RawFd) -> io::Result<()> {
    let mut block = [ParamRegW::raw_fd(fd), ParamRegW::uninit()];
    unsafe { syscall(OperationCode::HOSTED_CLOSE, ParamRegW::block(&mut block)) }
    let res = block[0].to_ret();
    if res.usize() == 0 {
        Ok(())
    } else {
        debug_assert_eq!(res.int(), -1);
        Err(from_errno(block[1].to_ret()))
    }
}
pub(crate) use self::hosted_close as close;

// TODO: Add uninit variant?
/// HOSTED_READ
pub fn hosted_read(fd: BorrowedFd<'_>, buf: &mut [u8]) -> io::Result<usize> {
    let len = buf.len();
    let mut block = [ParamRegW::fd(fd), ParamRegW::buf(buf), ParamRegW::usize(len)];
    unsafe { syscall(OperationCode::HOSTED_READ, ParamRegW::block(&mut block)) }
    let res = block[0].to_ret();
    if res.int() == -1 {
        Err(from_errno(block[1].to_ret()))
    } else {
        debug_assert!(res.usize() <= buf.len());
        Ok(res.usize())
    }
}
#[cfg(any(feature = "stdio", feature = "fs"))]
pub(crate) use self::hosted_read as read;

/// HOSTED_WRITE
pub fn hosted_write(fd: BorrowedFd<'_>, buf: &[u8]) -> io::Result<usize> {
    let mut block = [ParamRegW::fd(fd), ParamRegW::imm_buf(buf), ParamRegW::usize(buf.len())];
    unsafe { syscall(OperationCode::HOSTED_WRITE, ParamRegW::block(&mut block)) }
    let res = block[0].to_ret();
    if res.int() == -1 {
        Err(from_errno(block[1].to_ret()))
    } else {
        debug_assert!(res.usize() <= buf.len());
        Ok(res.usize())
    }
}
#[cfg(any(feature = "stdio", feature = "fs"))]
pub(crate) use self::hosted_write as write;

/// HOSTED_LSEEK
pub unsafe fn hosted_lseek(fd: BorrowedFd<'_>, offset: i64, flag: LseekFlag) -> io::Result<u64> {
    /// A 64-bit value represented as a pair of 32-bit values.
    ///
    /// This type is `#[repr(C)]`, both fields have the same in-memory representation
    /// and are plain old data types, so access to the fields is always safe.
    #[derive(Clone, Copy)]
    #[repr(C)]
    union I64 {
        i64: i64,
        u64: u64,
        pair: Pair,
    }
    #[derive(Clone, Copy)]
    #[repr(C)]
    struct Pair {
        // little endian order
        #[cfg(any(
            target_endian = "little",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "arm64ec",
        ))]
        lo: u32,
        hi: u32,
        // big endian order
        #[cfg(not(any(
            target_endian = "little",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "arm64ec",
        )))]
        lo: u32,
    }

    let offset = unsafe { I64 { i64: offset }.pair };
    let mut block = [
        ParamRegW::fd(fd),
        ParamRegW::usize(offset.hi as usize),
        ParamRegW::usize(offset.lo as usize),
        ParamRegW::usize(flag as usize),
    ];
    unsafe { syscall(OperationCode::HOSTED_LSEEK, ParamRegW::block(&mut block)) }
    let res_hi = block[0].to_ret();
    if res_hi.int() == -1 {
        Err(from_errno(block[2].to_ret()))
    } else {
        let res_lo = block[1].to_ret();
        Ok(unsafe {
            I64 { pair: Pair { hi: res_hi.usize() as u32, lo: res_lo.usize() as u32 } }.u64
        })
    }
}

/// HOSTED_RENAME
pub fn hosted_rename(old: &CStr, new: &CStr) -> io::Result<()> {
    let mut block = [
        ParamRegW::c_str(old),
        ParamRegW::c_str_len(old),
        ParamRegW::c_str(new),
        ParamRegW::c_str_len(new),
    ];
    unsafe { syscall(OperationCode::HOSTED_RENAME, ParamRegW::block(&mut block)) }
    let res = block[0].to_ret();
    if res.usize() == 0 {
        Ok(())
    } else {
        debug_assert_eq!(res.int(), -1);
        Err(from_errno(block[1].to_ret()))
    }
}

/// HOSTED_UNLINK
pub fn hosted_unlink(path: &CStr) -> io::Result<()> {
    let mut block = [ParamRegW::c_str(path), ParamRegW::c_str_len(path)];
    unsafe { syscall(OperationCode::HOSTED_UNLINK, ParamRegW::block(&mut block)) }
    let res = block[0].to_ret();
    if res.usize() == 0 {
        Ok(())
    } else {
        debug_assert_eq!(res.int(), -1);
        Err(from_errno(block[1].to_ret()))
    }
}

/// HOSTED_STAT
pub fn hosted_stat(path: &CStr) -> io::Result<stat> {
    let mut stat: stat = unsafe { mem::zeroed() };
    let mut block =
        [ParamRegW::c_str(path), ParamRegW::c_str_len(path), ParamRegW::ref_(&mut stat)];
    unsafe { syscall(OperationCode::HOSTED_STAT, ParamRegW::block(&mut block)) }
    let res = block[0].to_ret();
    if res.usize() == 0 {
        Ok(stat)
    } else {
        debug_assert_eq!(res.int(), -1);
        Err(from_errno(block[1].to_ret()))
    }
}

/// HOSTED_FSTAT
pub fn hosted_fstat(fd: BorrowedFd<'_>) -> io::Result<stat> {
    let mut stat: stat = unsafe { mem::zeroed() };
    let mut block = [ParamRegW::fd(fd), ParamRegW::ref_(&mut stat)];
    unsafe { syscall(OperationCode::HOSTED_FSTAT, ParamRegW::block(&mut block)) }
    let res = block[0].to_ret();
    if res.usize() == 0 {
        Ok(stat)
    } else {
        debug_assert_eq!(res.int(), -1);
        Err(from_errno(block[1].to_ret()))
    }
}

/// HOSTED_GETTIMEOFDAY
pub fn hosted_gettimeofday() -> io::Result<timeval> {
    let mut timeval: timeval = unsafe { mem::zeroed() };
    let mut block = [ParamRegW::ref_(&mut timeval), ParamRegW::uninit()];
    unsafe { syscall(OperationCode::HOSTED_GETTIMEOFDAY, ParamRegW::block(&mut block)) }
    let res = block[0].to_ret();
    if res.usize() == 0 {
        Ok(timeval)
    } else {
        debug_assert_eq!(res.int(), -1);
        Err(from_errno(block[1].to_ret()))
    }
}

/// HOSTED_ISATTY
pub fn hosted_isatty(fd: BorrowedFd<'_>) -> io::Result<bool> {
    let mut block = [ParamRegW::fd(fd), ParamRegW::uninit()];
    unsafe { syscall(OperationCode::HOSTED_ISATTY, ParamRegW::block(&mut block)) }
    let res = block[0].to_ret();
    match res.usize() {
        1 => Ok(true),
        0 => Ok(false),
        _ => Err(from_errno(block[1].to_ret())),
    }
}

/// HOSTED_SYSTEM
pub fn hosted_system(command: &CStr) -> io::Result<usize> {
    let mut block = [ParamRegW::c_str(command), ParamRegW::c_str_len(command)];
    unsafe { syscall(OperationCode::HOSTED_SYSTEM, ParamRegW::block(&mut block)) }
    let res = block[0].to_ret();
    // TODO
    if res.int() == -1 { Err(from_errno(block[1].to_ret())) } else { Ok(res.usize()) }
}
