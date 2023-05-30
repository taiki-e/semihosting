// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Raw semihosting call.

// TODO: should we expose this as public API of `sys` module?

#![allow(clippy::needless_pass_by_value)]

pub(crate) use arch::{syscall, syscall_readonly};
#[cfg_attr(target_arch = "aarch64", path = "aarch64.rs")]
#[cfg_attr(target_arch = "arm", path = "arm.rs")]
#[cfg_attr(any(target_arch = "riscv32", target_arch = "riscv64"), path = "riscv.rs")]
mod arch;

pub(crate) use crate::sys::reg::{ParamRegR, ParamRegW, RetReg};

/// Semihosting operation numbers.
///
/// - `0x00-0x31` Used by ARM.
/// - `0x32-0xFF` Reserved for future use by ARM.
/// - `0x100-0x1FF` Reserved for user applications.
/// - `0x200-0xFFFFFFFF` Undefined and currently unused.
#[repr(u32)]
pub(crate) enum OperationNumber {
    /// [SYS_OPEN (0x01)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#612sys_open-0x01)
    SYS_OPEN = 0x01,
    /// [SYS_CLOSE (0x02)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#62sys_close-0x02)
    SYS_CLOSE = 0x02,
    /// [SYS_WRITEC (0x03)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#623sys_writec-0x03)
    SYS_WRITEC = 0x03,
    /// [SYS_WRITE0 (0x04)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#624sys_write0-0x04)
    SYS_WRITE0 = 0x04,
    /// [SYS_WRITE (0x05)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#622sys_write-0x05)
    SYS_WRITE = 0x05,
    /// [SYS_READ (0x06)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#613sys_read-0x06)
    SYS_READ = 0x06,
    /// [SYS_READC (0x07)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#614sys_readc-0x07)
    SYS_READC = 0x07,
    /// [SYS_ISERROR (0x08)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#610sys_iserror-0x08)
    SYS_ISERROR = 0x08,
    /// [SYS_ISTTY (0x09)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#611sys_istty-0x09)
    SYS_ISTTY = 0x09,
    /// [SYS_SEEK (0x0A)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#617sys_seek-0x0a)
    SYS_SEEK = 0x0A,
    /// [SYS_FLEN (0x0C)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#67sys_flen-0x0c)
    SYS_FLEN = 0x0C,
    // /// [SYS_TMPNAM (0x0D)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#621sys_tmpnam-0x0d)
    // #[deprecated = "tmpnam is deprecated as not secure on most host systems"]
    //  SYS_TMPNAM = 0x0D,
    /// [SYS_REMOVE (0x0E)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#615sys_remove-0x0e)
    SYS_REMOVE = 0x0E,
    /// [SYS_RENAME (0x0F)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#616sys_rename-0x0f)
    SYS_RENAME = 0x0F,
    /// [SYS_CLOCK (0x10)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#sys-clock-0x10)
    SYS_CLOCK = 0x10,
    /// [SYS_TIME (0x11)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#620sys_time-0x11)
    SYS_TIME = 0x11,
    /// [SYS_SYSTEM (0x12)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#618sys_system-0x12)
    SYS_SYSTEM = 0x12,
    /// [SYS_ERRNO (0x13)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#64sys_errno-0x13)
    SYS_ERRNO = 0x13,
    /// [SYS_GET_CMDLINE (0x15)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#68sys_get_cmdline-0x15)
    SYS_GET_CMDLINE = 0x15,
    /// [SYS_HEAPINFO (0x16)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#69sys_heapinfo-0x16)
    SYS_HEAPINFO = 0x16,
    // #[deprecated = "obsoleted in semihosting specification version 2.0"]
    //  angel_SWIreason_EnterSVC = 0x17,
    /// [SYS_EXIT (0x18)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#65sys_exit-0x18)
    #[doc(alias = "angel_SWIreason_ReportException")] // old name
    SYS_EXIT = 0x18,
    // #[deprecated = "obsoleted in semihosting specification version 2.0"]
    //  angelSWI_Reason_SyncCacheRange = 0x19,
    /// [SYS_EXIT_EXTENDED (0x20)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#66sys_exit_extended-0x20)
    // we use SYS_EXIT on 64-bit system because SYS_EXIT_EXTENDED is identical to the behavior of the mandatory SYS_EXIT.
    #[cfg_attr(target_pointer_width = "64", allow(dead_code))]
    SYS_EXIT_EXTENDED = 0x20,
    /// [SYS_ELAPSED (0x30)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#63sys_elapsed-0x30)
    SYS_ELAPSED = 0x30,
    /// [SYS_TICKFREQ (0x31)](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst#619sys_tickfreq-0x31)
    SYS_TICKFREQ = 0x31,
}
pub(crate) use OperationNumber::*;

/// `syscall_readonly(number, null)`
#[inline]
pub(crate) unsafe fn syscall0(number: OperationNumber) -> RetReg {
    // In most operations that don't have parameters, such as SYS_ERRNO, and
    // SYS_CLOCK, the PARAMETER REGISTER must be zero.
    unsafe { syscall_readonly(number, ParamRegR::usize(0)) }
}
