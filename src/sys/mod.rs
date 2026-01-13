// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Low-level access to platform-specific semihosting interfaces.
//!
//! - At this (`semihosting::sys`) module level, wrappers around common semihosting interfaces
//!   across multiple semihosting interfaces are provided. Its functionality usually corresponds to the POSIX function of the same name, but is memory-safe and I/O-safe.
//!   - Note that we do not provide correspondents for POSIX functions that already have high-level APIs doing exactly the same thing:
//!     [`exit`](crate::process::exit), [`unlink`](crate::fs::remove_file), [`rename`](crate::fs::rename), [`isatty`](crate::io::IsTerminal::is_terminal)
//! - At the following platform-specific module level, thin wrappers around more platform-specific operations,
//!   including raw semihosting calls are provided. These usually just call the corresponding
//!   semihosting calls, but is memory-safe and I/O-safe.
//!   - `arm_compat`: AArch64, Arm, RISC-V, LoongArch, Xtensa (openocd-semihosting)
//!   - `mips`: MIPS32, MIPS64

#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    clippy::unnecessary_wraps,
    clippy::upper_case_acronyms
)]

#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv32",
    target_arch = "riscv64",
    target_arch = "loongarch32",
    target_arch = "loongarch64",
    all(target_arch = "xtensa", feature = "openocd-semihosting"),
))]
use self::arm_compat as arch;
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv32",
    target_arch = "riscv64",
    target_arch = "loongarch32",
    target_arch = "loongarch64",
    all(target_arch = "xtensa", feature = "openocd-semihosting"),
))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "loongarch32",
        target_arch = "loongarch64",
        all(target_arch = "xtensa", feature = "openocd-semihosting"),
    )))
)]
pub mod arm_compat;

#[cfg(any(
    target_arch = "mips",
    target_arch = "mips32r6",
    target_arch = "mips64",
    target_arch = "mips64r6",
))]
use self::mips as arch;
#[cfg(any(
    all(doc, docsrs),
    target_arch = "mips",
    target_arch = "mips32r6",
    target_arch = "mips64",
    target_arch = "mips64r6",
))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_arch = "mips",
        target_arch = "mips32r6",
        target_arch = "mips64",
        target_arch = "mips64r6",
    )))
)]
pub mod mips;

#[cfg(feature = "args")]
pub(crate) mod env;
mod errno;
#[cfg(feature = "random")]
pub(crate) mod random;
mod reg;
#[cfg(feature = "time")]
pub(crate) mod time;

use core::mem::MaybeUninit;

#[cfg(feature = "stdio")]
pub(crate) use self::arch::stdio;
#[cfg(feature = "fs")]
pub(crate) use self::arch::{errno::EINVAL, fs};
pub(crate) use self::{
    arch::exit,
    errno::{decode_error_kind, is_interrupted},
};
use crate::{
    fd::{BorrowedFd, RawFd},
    io,
};

/// Closes the specified file descriptor.
///
/// Note that [`OwnedFd`](crate::fd::OwnedFd) closes its file descriptor automatically on drop,
/// so this is unnecessary for most users.
///
/// # Safety
///
/// See [I/O Safety documentation](https://doc.rust-lang.org/std/io/index.html#io-safety).
///
/// # Platform-specific behavior
///
/// The following semihosting calls are currently being used:
///
/// | Platform                                                      | Semihosting call |
/// | ------------------------------------------------------------- | ---------------- |
/// | AArch64, Arm, RISC-V, LoongArch, Xtensa (openocd-semihosting) | [SYS_CLOSE]      |
/// | MIPS32, MIPS64                                                | UHI_close        |
///
/// [SYS_CLOSE]: https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-close-0x02
///
/// **Disclaimer:** These semihosting calls might change over time.
#[doc(alias = "SYS_CLOSE")] // arm_compat
#[doc(alias = "UHI_close")] // mips
pub unsafe fn close(fd: RawFd) -> io::Result<()> {
    // SAFETY: the caller must uphold the safety contract.
    unsafe { arch::close(fd) }
}

/// Reads to the specified buffer, returning how many bytes were read.
///
/// See also [`read_uninit`].
///
/// # Platform-specific behavior
///
/// The following semihosting calls are currently being used:
///
/// | Platform                                                      | Semihosting call |
/// | ------------------------------------------------------------- | ---------------- |
/// | AArch64, Arm, RISC-V, LoongArch, Xtensa (openocd-semihosting) | [SYS_READ]       |
/// | MIPS32, MIPS64                                                | UHI_read         |
///
/// [SYS_READ]: https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-read-0x06
///
/// **Disclaimer:** These semihosting calls might change over time.
#[doc(alias = "SYS_READ")] // arm_compat
#[doc(alias = "UHI_read")] // mips
pub fn read(fd: BorrowedFd<'_>, buf: &mut [u8]) -> io::Result<usize> {
    arch::read(fd, buf)
}

/// Reads to the specified buffer, returning the content that was read and the
/// remaining part of the buffer.
///
/// See also [`read`].
///
/// # Platform-specific behavior
///
/// See [`read`'s documentation](read#platform-specific-behavior).
#[doc(alias = "SYS_READ")] // arm_compat
#[doc(alias = "UHI_read")] // mips
pub fn read_uninit<'a>(
    fd: BorrowedFd<'_>,
    buf: &'a mut [MaybeUninit<u8>],
) -> io::Result<(&'a mut [u8], &'a mut [MaybeUninit<u8>])> {
    arch::read_uninit(fd, buf)
}

/// Writes from the specified buffer, returning how many bytes were written.
///
/// # Platform-specific behavior
///
/// The following semihosting calls are currently being used:
///
/// | Platform                                                      | Semihosting call |
/// | ------------------------------------------------------------- | ---------------- |
/// | AArch64, Arm, RISC-V, LoongArch, Xtensa (openocd-semihosting) | [SYS_WRITE]       |
/// | MIPS32, MIPS64                                                | UHI_write         |
///
/// [SYS_WRITE]: https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-write-0x05
///
/// **Disclaimer:** These semihosting calls might change over time.
#[doc(alias = "SYS_WRITE")] // arm_compat
#[doc(alias = "UHI_write")] // mips
pub fn write(fd: BorrowedFd<'_>, buf: &[u8]) -> io::Result<usize> {
    arch::write(fd, buf)
}
