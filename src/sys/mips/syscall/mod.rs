// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Raw semihosting call.

// TODO: should we expose this as public API of `sys` module?

#![allow(clippy::needless_pass_by_value)]

pub(crate) use arch::{
    syscall0, syscall1_readonly, syscall2, syscall2_readonly, syscall3, syscall3_readonly,
    syscall4, syscall4_readonly,
};
#[cfg_attr(
    any(
        doc,
        target_arch = "mips",
        target_arch = "mips32r6",
        target_arch = "mips64",
        target_arch = "mips64r6",
    ),
    path = "mips.rs"
)]
mod arch;

pub(crate) use crate::sys::reg::{ParamRegR, ParamRegW, RetReg};

/// Semihosting operation code.
#[derive(Clone, Copy)]
#[repr(usize)]
#[non_exhaustive]
pub(crate) enum OperationCode {
    UHI_exit = 1,
    UHI_open = 2,
    UHI_close = 3,
    UHI_read = 4,
    UHI_write = 5,
    UHI_lseek = 6,
    UHI_unlink = 7,
    UHI_fstat = 8,
    UHI_argc = 9,
    UHI_argnlen = 10,
    UHI_argn = 11,
    // UHI_ramrange = 12, // QEMU (as of 7.2) doesn't support this
    // UHI_plog = 13, // TODO
    // UHI_assert = 14, // TODO
    // UHI_exception = 15, // QEMU (as of 7.2) doesn't support this
    UHI_pread = 19,
    UHI_pwrite = 20,
    UHI_link = 22,
    // UHI_boot_fail = 23, // QEMU (as of 7.2) doesn't support this
}
pub(crate) use OperationCode::*;
