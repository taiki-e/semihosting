// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Low-level access to semihosting interfaces for Arm (AArch32 and AArch64) and
//! RISC-V (which supports Arm-compatible semihosting interfaces).
//!
//! Refs:
//! - Semihosting for AArch32 and AArch64 <https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst>
//! - RISC-V Semihosting <https://github.com/riscv-non-isa/riscv-semihosting/blob/1.0/riscv-semihosting.adoc>
//! - <https://github.com/qemu/qemu/blob/v10.2.0/semihosting/arm-compat-semi.c>
//! - <https://github.com/espressif/openocd-esp32/blob/HEAD/src/target/espressif/esp_xtensa_semihosting.c>

#![allow(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)] // TODO

pub(crate) mod errno;
#[cfg(feature = "fs")]
pub(crate) mod fs;
#[cfg(feature = "stdio")]
pub(crate) mod stdio;
pub mod syscall;

use core::{
    ffi::{CStr, c_void},
    mem::{self, MaybeUninit},
};

use self::syscall::{
    OperationNumber, ParamRegR, ParamRegW, syscall, syscall_noreturn_readonly,
    syscall_param_unchanged, syscall_param_unchanged_readonly, syscall_readonly,
    syscall0_param_unchanged,
};
use crate::{
    fd::{BorrowedFd, OwnedFd, RawFd},
    io,
    utils::slice_assume_init_mut,
};

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
#[repr(usize)]
#[non_exhaustive]
pub enum ExitReason {
    // Reason codes related to hardware exceptions:
    ADP_Stopped_BranchThroughZero = 0x20000,
    ADP_Stopped_UndefinedInstr = 0x20001,
    ADP_Stopped_SoftwareInterrupt = 0x20002,
    ADP_Stopped_PrefetchAbort = 0x20003,
    ADP_Stopped_DataAbort = 0x20004,
    ADP_Stopped_AddressException = 0x20005,
    ADP_Stopped_IRQ = 0x20006,
    ADP_Stopped_FIQ = 0x20007,

    // Reason codes related to software events:
    ADP_Stopped_BreakPoint = 0x20020,
    ADP_Stopped_WatchPoint = 0x20021,
    ADP_Stopped_StepComplete = 0x20022,
    ADP_Stopped_RunTimeErrorUnknown = 0x20023,
    ADP_Stopped_InternalError = 0x20024,
    ADP_Stopped_UserInterruption = 0x20025,
    ADP_Stopped_ApplicationExit = 0x20026,
    ADP_Stopped_StackOverflow = 0x20027,
    ADP_Stopped_DivisionByZero = 0x20028,
    ADP_Stopped_OSSpecific = 0x20029,
}

// Refs: https://github.com/openocd-org/openocd/blob/HEAD/src/target/semihosting_common.c
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
#[repr(usize)]
#[non_exhaustive]
pub enum OpenMode {
    /// `fopen` mode: `r`<br>
    /// `O_*` flags: `O_RDONLY`
    RDONLY = 0,
    /// `fopen` mode: `rb`<br>
    /// `O_*` flags: `O_RDONLY | O_BINARY`
    RDONLY_BINARY = 1,
    /// `fopen` mode: `r+`<br>
    /// `O_*` flags: `O_RDWR`
    RDWR = 2,
    /// `fopen` mode: `r+b`<br>
    /// `O_*` flags: `O_RDWR | O_BINARY`
    RDWR_BINARY = 3,
    /// `fopen` mode: `w`<br>
    /// `O_*` flags: `O_WRONLY | O_CREAT | O_TRUNC`
    WRONLY_TRUNC = 4,
    /// `fopen` mode: `wb`<br>
    /// `O_*` flags: `O_WRONLY | O_CREAT | O_TRUNC | O_BINARY`
    WRONLY_TRUNC_BINARY = 5,
    /// `fopen` mode: `w+`<br>
    /// `O_*` flags: `O_RDWR | O_CREAT | O_TRUNC`
    RDWR_TRUNC = 6,
    /// `fopen` mode: `w+b`<br>
    /// `O_*` flags: `O_RDWR | O_CREAT | O_TRUNC | O_BINARY`
    RDWR_TRUNC_BINARY = 7,
    /// `fopen` mode: `a`<br>
    /// `O_*` flags: `O_WRONLY | O_CREAT | O_APPEND`
    WRONLY_APPEND = 8,
    /// `fopen` mode: `ab`<br>
    /// `O_*` flags: `O_WRONLY | O_CREAT | O_APPEND | O_BINARY`
    WRONLY_APPEND_BINARY = 9,
    /// `fopen` mode: `a+`<br>
    /// `O_*` flags: `O_RDWR | O_CREAT | O_APPEND`
    RDWR_APPEND = 10,
    /// `fopen` mode: `a+b`<br>
    /// `O_*` flags: `O_RDWR | O_CREAT | O_APPEND | O_BINARY`
    RDWR_APPEND_BINARY = 11,
}

#[allow(missing_docs)]
#[allow(clippy::exhaustive_structs)] // TODO(semver)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct HeapInfo {
    pub heap_base: *mut c_void,
    pub heap_limit: *mut c_void,
    pub stack_base: *mut c_void,
    pub stack_limit: *mut c_void,
}

// TODO(semver): Remove
#[allow(missing_docs)]
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CommandLine {
    pub ptr: *mut u8,
    pub size: usize,
}

#[cold]
fn from_errno() -> io::Error {
    io::Error::from_raw_os_error(sys_errno())
}

/// [SYS_CLOCK (0x10)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-clock-0x10)
pub fn sys_clock() -> io::Result<usize> {
    // |                    | on success      | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | other           | -1              |                 |
    // | PARAMETER REGISTER | - (unmentioned) | - (unmentioned) | param_unchanged |
    let ret = unsafe { syscall0_param_unchanged(OperationNumber::SYS_CLOCK) };
    if ret.signed() == -1 { Err(from_errno()) } else { Ok(ret.unsigned()) }
}

/// [SYS_CLOSE (0x02)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-close-0x02)
/// (Equivalent to [`sys::close`](crate::sys::close))
pub unsafe fn sys_close(fd: RawFd) -> io::Result<()> {
    let block = [ParamRegR::raw_fd(fd)];
    // |                    | on success      | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | 0               | -1              |                 |
    // | PARAMETER REGISTER | - (unmentioned) | - (unmentioned) | param_unchanged |
    // | block              | - (unmentioned) | - (unmentioned) | readonly        |
    let ret = unsafe {
        syscall_param_unchanged_readonly(OperationNumber::SYS_CLOSE, ParamRegR::block(&block))
    };
    if ret.unsigned() == 0 {
        Ok(())
    } else {
        debug_assert_eq!(ret.signed(), -1);
        Err(from_errno())
    }
}
pub(crate) use self::sys_close as close;

/// [SYS_ELAPSED (0x30)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-elapsed-0x30)
pub fn sys_elapsed() -> io::Result<u64> {
    // On 32-bit, the parameter is a pointer to two 32-bit field data block
    // On 64-bit, the parameter is a pointer to one 64-bit field data block
    let mut block = [0_u64];
    // |                    | on success      | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | 0               | -1              |                 |
    // | PARAMETER REGISTER | - (unchanged)   | -1              |                 |
    // | block              | updated         | - (unmentioned) |                 |
    let ret = unsafe { syscall(OperationNumber::SYS_ELAPSED, ParamRegW::ref_(&mut block)) };
    if ret.unsigned() == 0 {
        Ok(block[0])
    } else {
        debug_assert_eq!(ret.signed(), -1);
        Err(from_errno())
    }
}

/// [SYS_ERRNO (0x13)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-errno-0x13)
pub fn sys_errno() -> io::RawOsError {
    // |                    | always          |                 |
    // | ------------------ | --------------- | --------------- |
    // | RETURN REGISTER    | updated         |                 |
    // | PARAMETER REGISTER | - (unmentioned) | param_unchanged |
    let ret = unsafe { syscall0_param_unchanged(OperationNumber::SYS_ERRNO) };
    ret.errno()
}

/// [SYS_EXIT (0x18)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-exit-0x18)
// TODO(semver): change return type to !?
pub fn sys_exit(reason: ExitReason) {
    #[cfg(target_pointer_width = "32")]
    let arg = ParamRegR::unsigned(reason as usize);
    #[cfg(target_pointer_width = "64")]
    let block = [ParamRegR::unsigned(reason as usize), ParamRegR::unsigned(0)];
    #[cfg(target_pointer_width = "64")]
    let arg = ParamRegR::block(&block);
    // > No return is expected from these calls. However, it is possible for the
    // > debugger to request that the application continues by performing an
    // > RDI_Execute request or equivalent. In this case, execution continues
    // > with the registers as they were on entry to the operation, or as
    // > subsequently modified by the debugger.
    // |                    | always          |                 |
    // | ------------------ | --------------- | --------------- |
    // | RETURN REGISTER    | ?               |                 |
    // | PARAMETER REGISTER | ?               |                 |
    // | block              | - (unmentioned) | readonly        |
    unsafe {
        syscall_readonly(OperationNumber::SYS_EXIT, arg);
    }
}
pub(crate) fn exit(code: i32) -> ! {
    let reason = ExitReason::ADP_Stopped_ApplicationExit;
    #[allow(clippy::cast_sign_loss)]
    let subcode = code as isize as usize;
    // On 64-bit system, SYS_EXIT_EXTENDED call is identical to the behavior of the mandatory SYS_EXIT.
    #[cfg(target_pointer_width = "64")]
    unsafe {
        let block = [ParamRegR::unsigned(reason as usize), ParamRegR::unsigned(subcode)];
        syscall_noreturn_readonly(OperationNumber::SYS_EXIT, ParamRegR::block(&block))
    }
    #[cfg(target_pointer_width = "32")]
    {
        // TODO: check sh_ext_exit_extended first
        sys_exit_extended(reason, subcode);

        // If SYS_EXIT_EXTENDED is not supported, above call doesn't exit program, so call SYS_EXIT.
        let reason = match code {
            0 => ExitReason::ADP_Stopped_ApplicationExit,
            _ => ExitReason::ADP_Stopped_RunTimeErrorUnknown,
        };
        unsafe {
            syscall_noreturn_readonly(
                OperationNumber::SYS_EXIT,
                ParamRegR::unsigned(reason as usize),
            )
        }
    }
}

/// [SYS_EXIT_EXTENDED (0x20)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-exit-extended-0x20)
pub fn sys_exit_extended(reason: ExitReason, subcode: usize) {
    let block = [ParamRegR::unsigned(reason as usize), ParamRegR::unsigned(subcode)];
    #[cfg(target_pointer_width = "32")]
    let number = OperationNumber::SYS_EXIT_EXTENDED;
    // On 64-bit system, SYS_EXIT_EXTENDED call is identical to the behavior of the mandatory SYS_EXIT.
    #[cfg(target_pointer_width = "64")]
    let number = OperationNumber::SYS_EXIT;
    // |                    | always          |                 |
    // | ------------------ | --------------- | --------------- |
    // | RETURN REGISTER    | ?               |                 |
    // | PARAMETER REGISTER | ?               |                 |
    // | block              | - (unmentioned) | readonly        |
    unsafe {
        syscall_readonly(number, ParamRegR::block(&block));
    }
}

/// [SYS_FLEN (0x0C)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-flen-0x0c)
pub fn sys_flen(fd: BorrowedFd<'_>) -> io::Result<usize> {
    let block = [ParamRegR::fd(fd)];
    // |                    | on success      | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | other           | -1              |                 |
    // | PARAMETER REGISTER | - (unmentioned) | - (unmentioned) | param_unchanged |
    // | block              | - (unmentioned) | - (unmentioned) | readonly        |
    let ret = unsafe {
        syscall_param_unchanged_readonly(OperationNumber::SYS_FLEN, ParamRegR::block(&block))
    };
    if ret.signed() == -1 { Err(from_errno()) } else { Ok(ret.unsigned()) }
}

/// Deprecated: use [`sys_get_cmdline_uninit`] instead
///
/// # Safety
///
/// CommandLine::ptr must be valid for at least the size specified in CommandLine::size.
#[deprecated(note = "use safer [`sys_get_cmdline_uninit`] instead")] // TODO(semver): Remove
pub unsafe fn sys_get_cmdline(cmdline: &mut CommandLine) -> io::Result<()> {
    let len = cmdline.size;
    // |                    | on success      | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | 0               | -1              |                 |
    // | PARAMETER REGISTER | - (unchanged)   | - (unmentioned) | param_unchanged |
    // | cmdline / ptr      | updated         | - (unmentioned) |                 |
    let ret = unsafe {
        syscall_param_unchanged(OperationNumber::SYS_GET_CMDLINE, ParamRegW::ref_(cmdline))
    };
    if ret.unsigned() == 0 {
        debug_assert!(!cmdline.ptr.is_null());
        let size = cmdline.size;
        debug_assert!(size < len); // len contains trailing nul
        Ok(())
    } else {
        debug_assert_eq!(ret.signed(), -1);
        Err(from_errno())
    }
}

/// [SYS_GET_CMDLINE (0x15)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-get-cmdline-0x15)
pub fn sys_get_cmdline_uninit(buf: &mut [MaybeUninit<u8>]) -> io::Result<&mut [u8]> {
    let len = buf.len();
    let mut block = [ParamRegW::buf(buf), ParamRegW::unsigned(len)];
    // |                    | on success      | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | 0               | -1              |                 |
    // | PARAMETER REGISTER | - (unchanged)   | - (unmentioned) | param_unchanged |
    // | block / buf        | updated         | - (unmentioned) |                 |
    let ret = unsafe {
        syscall_param_unchanged(OperationNumber::SYS_GET_CMDLINE, ParamRegW::block(&mut block))
    };
    if ret.unsigned() == 0 {
        debug_assert!(!block[0].to_ret().ptr().is_null());
        let size = block[1].to_ret().unsigned();
        debug_assert!(size < len); // len contains trailing nul
        Ok(unsafe { slice_assume_init_mut(&mut buf[..size]) })
    } else {
        debug_assert_eq!(ret.signed(), -1);
        Err(from_errno())
    }
}

/// [SYS_HEAPINFO (0x16)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-heapinfo-0x16)
pub fn sys_heapinfo() -> HeapInfo {
    let mut buf: HeapInfo = unsafe { mem::zeroed() };
    // |                    | on success      | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | ? (unmentioned) | ? (unmentioned) |                 |
    // | PARAMETER REGISTER | - (unchanged)   | - (unmentioned) | param_unchanged |
    // | buf                | updated         | - (unmentioned) |                 |
    unsafe {
        syscall_param_unchanged(OperationNumber::SYS_HEAPINFO, ParamRegW::ref_(&mut buf));
    }
    buf
}

/// [SYS_ISERROR (0x08)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-iserror-0x08)
pub fn sys_iserror(res: isize) -> bool {
    let block = [ParamRegR::signed(res)];
    // |                    | is not an error | is an error     |                 |
    // | ------------------ | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | 0               | nonzero         |                 |
    // | PARAMETER REGISTER | - (unmentioned) | - (unmentioned) | param_unchanged |
    // | block              | - (unmentioned) | - (unmentioned) | readonly        |
    let ret = unsafe {
        syscall_param_unchanged_readonly(OperationNumber::SYS_ISERROR, ParamRegR::block(&block))
    };
    ret.unsigned() != 0
}

/// [SYS_ISTTY (0x09)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-istty-0x09)
pub fn sys_istty(fd: BorrowedFd<'_>) -> io::Result<bool> {
    let block = [ParamRegR::fd(fd)];
    // |                    | is a tty        | is a file       | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | 1               | 0               | other           |                 |
    // | PARAMETER REGISTER | - (unmentioned) | - (unmentioned) | - (unmentioned) | param_unchanged |
    // | block              | - (unmentioned) | - (unmentioned) | - (unmentioned) | readonly        |
    let ret = unsafe {
        syscall_param_unchanged_readonly(OperationNumber::SYS_ISTTY, ParamRegR::block(&block))
    };
    match ret.unsigned() {
        1 => Ok(true),
        0 => Ok(false),
        _ => Err(from_errno()), // TODO: some host system doesn't set errno
    }
}

/// [SYS_OPEN (0x01)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-open-0x01)
pub fn sys_open(path: &CStr, mode: OpenMode) -> io::Result<OwnedFd> {
    let block =
        [ParamRegR::c_str(path), ParamRegR::unsigned(mode as usize), ParamRegR::c_str_len(path)];
    // |                    | on success      | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | nonzero         | -1              |                 |
    // | PARAMETER REGISTER | - (unmentioned) | - (unmentioned) | param_unchanged |
    // | block / path       | - (unmentioned) | - (unmentioned) | readonly        |
    let ret = unsafe {
        syscall_param_unchanged_readonly(OperationNumber::SYS_OPEN, ParamRegR::block(&block))
    };
    match ret.raw_fd() {
        Some(fd) => {
            debug_assert_ne!(ret.unsigned(), 0);
            Ok(unsafe { OwnedFd::from_raw_fd(fd) })
        }
        None => Err(from_errno()),
    }
}

/// Deprecated: use [`sys::read`](crate::sys::read), [`sys::read_uninit`](crate::sys::read_uninit), or [`sys_read_orig`] instead
///
///
/// **Note:** Unlike SYS_READ's original behavior, this returns the number of bytes read.
/// - If you want to continue to use uninitialized buffer, use [`sys::read_uninit`](crate::sys::read_uninit) instead.
/// - If you want the current behavior, use [`sys::read`](crate::sys::read) instead.
/// - If you want the SYS_READ's original behavior, use [`sys_read_orig`] instead.
///
/// This function may be changed to behave as [`sys_read_orig`] in a future breaking release.
#[deprecated(
    note = "use [`sys::read`](crate::sys::read), [`sys::read_uninit`](crate::sys::read_uninit), or [`sys_read_orig`] instead"
)] // TODO(semver)
pub fn sys_read(fd: BorrowedFd<'_>, buf: &mut [MaybeUninit<u8>]) -> io::Result<usize> {
    Ok(read_uninit(fd, buf)?.0.len())
}
/// [SYS_READ (0x06)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-read-0x06)
pub fn sys_read_orig(fd: BorrowedFd<'_>, buf: &mut [u8]) -> io::Result<usize> {
    let num_read = read(fd, buf)?;
    Ok(buf.len() - num_read)
}
pub(crate) fn read(fd: BorrowedFd<'_>, buf: &mut [u8]) -> io::Result<usize> {
    let len = buf.len();
    // SAFETY: transmuting initialized `&mut [u8]` to `&mut [MaybeUninit<u8>]` is safe unless uninitialized byte will be written to resulting slice.
    let buf =
        unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr().cast::<MaybeUninit<u8>>(), len) };
    Ok(read_uninit(fd, buf)?.0.len())
}
pub(crate) fn read_uninit<'a>(
    fd: BorrowedFd<'_>,
    buf: &'a mut [MaybeUninit<u8>],
) -> io::Result<(&'a mut [u8], &'a mut [MaybeUninit<u8>])> {
    let len = buf.len();
    let mut block = [ParamRegW::fd(fd), ParamRegW::buf(buf), ParamRegW::unsigned(len)];
    // |                    | fully filled    | on EOF          | partly filled   | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | 0               | == len          | < len           | > len || < 0    |                 |
    // | PARAMETER REGISTER | - (unmentioned) | - (unmentioned) | - (unmentioned) | - (unmentioned) | param_unchanged |
    // | block / buf        | updated         | - (unmentioned) | updated         | ? (unmentioned) |                 |
    let ret =
        unsafe { syscall_param_unchanged(OperationNumber::SYS_READ, ParamRegW::block(&mut block)) };
    let not_read = ret.unsigned();
    if not_read <= len {
        let (filled, rest) = buf.split_at_mut(buf.len() - not_read);
        Ok((unsafe { slice_assume_init_mut(filled) }, rest))
    } else {
        debug_assert!(ret.signed().is_negative());
        Err(from_errno())
    }
}

/// [SYS_READC (0x07)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-readc-0x07)
pub fn sys_readc() -> u8 {
    // |                    | always          |                 |
    // | ------------------ | --------------- | --------------- |
    // | RETURN REGISTER    | updated         |                 |
    // | PARAMETER REGISTER | - (unmentioned) | param_unchanged |
    let ret = unsafe { syscall0_param_unchanged(OperationNumber::SYS_READC) };
    ret.u8()
}

/// [SYS_REMOVE (0x0E)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-remove-0x0e)
/// (Equivalent to [`fs::remove_file`](crate::fs::remove_file))
pub fn sys_remove(path: &CStr) -> io::Result<()> {
    let block = [ParamRegR::c_str(path), ParamRegR::c_str_len(path)];
    // |                    | on success      | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | 0               | nonzero         |                 |
    // | PARAMETER REGISTER | - (unmentioned) | - (unmentioned) | param_unchanged |
    // | block / path       | - (unmentioned) | - (unmentioned) | readonly        |
    let ret = unsafe {
        syscall_param_unchanged_readonly(OperationNumber::SYS_REMOVE, ParamRegR::block(&block))
    };
    if ret.unsigned() == 0 { Ok(()) } else { Err(from_errno()) }
}

/// [SYS_RENAME (0x0F)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-rename-0x0f)
/// (Equivalent to [`fs::rename`](crate::fs::rename))
pub fn sys_rename(from: &CStr, to: &CStr) -> io::Result<()> {
    let block = [
        ParamRegR::c_str(from),
        ParamRegR::c_str_len(from),
        ParamRegR::c_str(to),
        ParamRegR::c_str_len(to),
    ];
    // |                    | on success      | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | 0               | nonzero         |                 |
    // | PARAMETER REGISTER | - (unmentioned) | - (unmentioned) | param_unchanged |
    // | block / from / to  | - (unmentioned) | - (unmentioned) | readonly        |
    let ret = unsafe {
        syscall_param_unchanged_readonly(OperationNumber::SYS_RENAME, ParamRegR::block(&block))
    };
    if ret.unsigned() == 0 { Ok(()) } else { Err(from_errno()) }
}

// TODO(arm_compat): resolve safety
// > The effect of seeking outside the current extent of the file object is undefined.
/// [SYS_SEEK (0x0A)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-seek-0x0a)
pub unsafe fn sys_seek(fd: BorrowedFd<'_>, abs_pos: usize) -> io::Result<()> {
    let block = [ParamRegR::fd(fd), ParamRegR::unsigned(abs_pos)];
    // |                    | on success      | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | 0               | negative        |                 |
    // | PARAMETER REGISTER | - (unmentioned) | - (unmentioned) | param_unchanged |
    // | block              | - (unmentioned) | - (unmentioned) | readonly        |
    let ret = unsafe {
        syscall_param_unchanged_readonly(OperationNumber::SYS_SEEK, ParamRegR::block(&block))
    };
    if ret.unsigned() == 0 {
        Ok(())
    } else {
        debug_assert!(ret.signed().is_negative());
        Err(from_errno())
    }
}

/// [SYS_SYSTEM (0x12)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-system-0x12)
pub fn sys_system(cmd: &CStr) -> usize {
    // On exit, the RETURN REGISTER contains the return status.
    // |                    | always          |                 |
    // | ------------------ | --------------- | --------------- |
    // | RETURN REGISTER    | updated         |                 |
    // | PARAMETER REGISTER | - (unmentioned) | param_unchanged |
    // | block / cmd        | - (unmentioned) | readonly        |
    let block = [ParamRegR::c_str(cmd), ParamRegR::c_str_len(cmd)];
    let ret = unsafe {
        syscall_param_unchanged_readonly(OperationNumber::SYS_SYSTEM, ParamRegR::block(&block))
    };
    ret.unsigned()
}

/// [SYS_TICKFREQ (0x31)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-tickfreq-0x31)
pub fn sys_tickfreq() -> io::Result<usize> {
    // |                    | on success      | on failure      |                 |
    // | ------------------ | --------------- | --------------- | --------------- |
    // | RETURN REGISTER    | other           | -1              |                 |
    // | PARAMETER REGISTER | - (unmentioned) | - (unmentioned) | param_unchanged |
    let ret = unsafe { syscall0_param_unchanged(OperationNumber::SYS_TICKFREQ) };
    if ret.signed() == -1 { Err(from_errno()) } else { Ok(ret.unsigned()) }
}

/// [SYS_TIME (0x11)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-time-0x11)
#[allow(clippy::unnecessary_wraps)] // TODO(semver): change in next breaking release
pub fn sys_time() -> io::Result<usize> {
    // |                    | always          |                 |
    // | ------------------ | --------------- | --------------- |
    // | RETURN REGISTER    | updated         |                 |
    // | PARAMETER REGISTER | - (unmentioned) | param_unchanged |
    let ret = unsafe { syscall0_param_unchanged(OperationNumber::SYS_TIME) };
    Ok(ret.unsigned())
}

/// Deprecated: use [`sys::write`](crate::sys::write) or [`sys_write_orig`] instead
///
/// **Note:** Unlike SYS_WRITE's original behavior, this returns the number of bytes written.
/// - If you want the current behavior, use [`sys::write`](crate::sys::write) instead.
/// - If you want the SYS_WRITE's original behavior, use [`sys_write_orig`] instead.
///
/// This function may be changed to behave as [`sys_write_orig`] in a future breaking release.
#[deprecated(note = "use [`sys::write`](crate::sys::write) or [`sys_write_orig`] instead")] // TODO(semver)
pub fn sys_write(fd: BorrowedFd<'_>, buf: &[u8]) -> io::Result<usize> {
    write(fd, buf)
}
/// [SYS_WRITE (0x05)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-write-0x05)
pub fn sys_write_orig(fd: BorrowedFd<'_>, buf: &[u8]) -> io::Result<usize> {
    let len = buf.len();
    let block = [ParamRegR::fd(fd), ParamRegR::buf(buf), ParamRegR::unsigned(len)];
    // |                    | fully filled    | partly filled / on failure |                 |
    // | ------------------ | --------------- | -------------------------- | --------------- |
    // | RETURN REGISTER    | 0               | nonzero                    |                 |
    // | PARAMETER REGISTER | - (unmentioned) | - (unmentioned)            | param_unchanged |
    // | block / buf        | - (unmentioned) | - (unmentioned)            | readonly        |
    let ret = unsafe {
        syscall_param_unchanged_readonly(OperationNumber::SYS_WRITE, ParamRegR::block(&block))
    };
    let not_written = ret.unsigned();
    #[allow(clippy::comparison_chain)]
    if not_written < len {
        Ok(not_written)
    } else if not_written == len {
        if len == 0 {
            Ok(not_written)
        } else {
            // At least on qemu-system-arm 7.2, if fd is a read-only file,
            // this is returned instead of an error.
            Err(from_errno())
        }
    } else {
        debug_assert!(ret.signed().is_negative());
        Err(from_errno())
    }
}
pub(crate) fn write(fd: BorrowedFd<'_>, buf: &[u8]) -> io::Result<usize> {
    let not_written = sys_write_orig(fd, buf)?;
    Ok(buf.len() - not_written)
}

/// [SYS_WRITEC (0x03)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-writec-0x03)
pub fn sys_writec(character: u8) {
    // |                    | always          |                 |
    // | ------------------ | --------------- | --------------- |
    // | RETURN REGISTER    | corrupted       |                 |
    // | PARAMETER REGISTER | - (unmentioned) | param_unchanged |
    // | character          | - (unmentioned) | readonly        |
    unsafe {
        syscall_param_unchanged_readonly(OperationNumber::SYS_WRITEC, ParamRegR::ref_(&character));
    }
}

/// [SYS_WRITE0 (0x04)](https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-write0-0x04)
pub fn sys_write0(string: &CStr) {
    // |                    | always          |                 |
    // | ------------------ | --------------- | --------------- |
    // | RETURN REGISTER    | corrupted       |                 |
    // | PARAMETER REGISTER | - (unmentioned) | param_unchanged |
    // | string             | - (unmentioned) | readonly        |
    unsafe {
        syscall_param_unchanged_readonly(OperationNumber::SYS_WRITE0, ParamRegR::c_str(string));
    }
}
