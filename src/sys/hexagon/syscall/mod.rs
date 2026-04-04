// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Raw semihosting call.

#[allow(clippy::needless_pass_by_value)]
#[cfg_attr(any(doc, target_arch = "hexagon"), path = "hexagon.rs")]
mod arch;

pub(crate) use self::arch::syscall_noreturn_readonly;
pub use self::arch::{direct_syscall, direct_syscall_readonly, syscall, syscall_readonly};
pub use crate::sys::reg::{ParamRegR, ParamRegW, RetReg};

/// Semihosting operation number.
#[derive(Debug, Clone, Copy)]
pub struct OperationNumber(u32);
#[allow(missing_docs)]
impl OperationNumber {
    pub const SYS_OPEN: Self = Self(0x01);
    pub const SYS_CLOSE: Self = Self(0x02);
    pub const SYS_WRITEC: Self = Self(0x03);
    pub const SYS_WRITE0: Self = Self(0x04);
    pub const SYS_WRITE: Self = Self(0x05);
    pub const SYS_READ: Self = Self(0x06);
    pub const SYS_READC: Self = Self(0x07);
    pub const SYS_ISERROR: Self = Self(0x08);
    pub const SYS_ISTTY: Self = Self(0x09);
    pub const SYS_SEEK: Self = Self(0x0A);
    pub const SYS_FLEN: Self = Self(0x0C);
    // #[deprecated = "tmpnam is deprecated as not secure on most host systems"]
    // pub const SYS_TMPNAM : Self = Self(0x0D);
    pub const SYS_REMOVE: Self = Self(0x0E);
    pub const SYS_RENAME: Self = Self(0x0F);
    pub const SYS_CLOCK: Self = Self(0x10);
    pub const SYS_TIME: Self = Self(0x11);
    pub const SYS_SYSTEM: Self = Self(0x12);
    pub const SYS_ERRNO: Self = Self(0x13);
    pub const SYS_GET_CMDLINE: Self = Self(0x15);
    pub const SYS_HEAPINFO: Self = Self(0x16);
    pub const SYS_EXCEPTION: Self = Self(0x18);
    pub const SYS_EXIT_EXTENDED: Self = Self(0x20);
    pub const SYS_ELAPSED: Self = Self(0x30);
    pub const SYS_TICKFREQ: Self = Self(0x31);
    pub const SYS_READ_CYCLES: Self = Self(0x40);
    pub const SYS_PROF_ON: Self = Self(0x41);
    pub const SYS_PROF_OFF: Self = Self(0x42);
    pub const SYS_WRITECREG: Self = Self(0x43);
    pub const SYS_READ_TCYCLES: Self = Self(0x44);
    pub const SYS_LOG_EVENT: Self = Self(0x45);
    pub const SYS_REDRAW: Self = Self(0x46);
    pub const SYS_READ_ICOUNT: Self = Self(0x47);
    pub const SYS_PROF_STATSRESET: Self = Self(0x48);
    pub const SYS_DUMP_PMU_STATS: Self = Self(0x4A);
    pub const SYS_READ_PCYCLES: Self = Self(0x52);
    pub const SYS_COREDUMP: Self = Self(0xCD);
    pub const SYS_FTELL: Self = Self(0x100);
    pub const SYS_FSTAT: Self = Self(0x101);
    pub const SYS_STAT: Self = Self(0x103);
    pub const SYS_GETCWD: Self = Self(0x104);
    pub const SYS_ACCESS: Self = Self(0x105);
    pub const SYS_OPENDIR: Self = Self(0x180);
    pub const SYS_CLOSEDIR: Self = Self(0x181);
    pub const SYS_READDIR: Self = Self(0x182);
    pub const SYS_EXEC: Self = Self(0x185);
    pub const SYS_FTRUNC: Self = Self(0x186);
}
