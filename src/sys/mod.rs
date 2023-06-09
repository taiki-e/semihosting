// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Low-level access to platform-specific semihosting interfaces.

#![allow(
    missing_debug_implementations,
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
))]
use arm_compat as arch;
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv32",
    target_arch = "riscv64",
))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "riscv32",
        target_arch = "riscv64",
    )))
)]
pub mod arm_compat;

#[cfg(any(target_arch = "mips", target_arch = "mips64"))]
use mips as arch;
#[cfg(any(all(doc, docsrs), target_arch = "mips", target_arch = "mips64"))]
#[cfg_attr(docsrs, doc(cfg(any(target_arch = "mips", target_arch = "mips64"))))]
pub mod mips;

mod errno;
mod reg;

#[cfg(feature = "fs")]
pub(crate) use arch::fs;
pub(crate) use arch::{close, exit, should_close};
#[cfg(feature = "stdio")]
pub(crate) use arch::{is_terminal, stderr, stdin, stdout, StdioFd};
#[cfg(any(feature = "stdio", feature = "fs"))]
pub(crate) use arch::{read, write};
pub(crate) use errno::decode_error_kind;
