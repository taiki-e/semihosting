// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Raw semihosting call.

pub use self::arch::{syscall, syscall_readonly};
#[allow(clippy::needless_pass_by_value)]
#[cfg_attr(target_arch = "m68k", path = "m68k.rs")]
mod arch;

pub use crate::sys::reg::{ParamRegR, ParamRegW, RetReg};

/// Semihosting operation code.
#[derive(Debug, Clone, Copy)]
pub struct OperationCode(u32);
#[allow(missing_docs)]
impl OperationCode {
    pub const HOSTED_EXIT: Self = Self(0);
    pub const HOSTED_INIT_SIM: Self = Self(1);
    pub const HOSTED_OPEN: Self = Self(2);
    pub const HOSTED_CLOSE: Self = Self(3);
    pub const HOSTED_READ: Self = Self(4);
    pub const HOSTED_WRITE: Self = Self(5);
    pub const HOSTED_LSEEK: Self = Self(6);
    pub const HOSTED_RENAME: Self = Self(7);
    pub const HOSTED_UNLINK: Self = Self(8);
    pub const HOSTED_STAT: Self = Self(9);
    pub const HOSTED_FSTAT: Self = Self(10);
    pub const HOSTED_GETTIMEOFDAY: Self = Self(11);
    pub const HOSTED_ISATTY: Self = Self(12);
    pub const HOSTED_SYSTEM: Self = Self(13);
}
