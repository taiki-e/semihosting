// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::ffi::c_int;

use crate::{
    io::{self, RawOsError},
    sys::arch::errno,
};

#[inline]
pub(crate) fn is_interrupted(errno: RawOsError) -> bool {
    errno as c_int == errno::EINTR
}

#[cfg(feature = "fs")]
#[cold]
pub(crate) fn einval() -> io::Error {
    io::Error::from_raw_os_error(errno::EINVAL)
}

// Adapted from https://github.com/rust-lang/rust/blob/1.92.0/library/std/src/sys/pal/unix/mod.rs#L235.
pub(crate) fn decode_error_kind(errno: RawOsError) -> io::ErrorKind {
    #[allow(clippy::enum_glob_use)]
    use io::ErrorKind::*;
    match errno as c_int {
        #[cfg(not(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        )))]
        errno::E2BIG => ArgumentListTooLong,
        // errno::EADDRINUSE => AddrInUse,
        // errno::EADDRNOTAVAIL => AddrNotAvailable,
        errno::EBUSY => ResourceBusy,
        // errno::ECONNABORTED => ConnectionAborted,
        // errno::ECONNREFUSED => ConnectionRefused,
        #[cfg(not(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "loongarch32",
            target_arch = "loongarch64",
            all(target_arch = "xtensa", feature = "openocd-semihosting"),
        )))]
        errno::ECONNRESET => ConnectionReset,
        // errno::EDEADLK => Deadlock,
        // errno::EDQUOT => QuotaExceeded,
        errno::EEXIST => AlreadyExists,
        errno::EFBIG => FileTooLarge,
        // errno::EHOSTUNREACH => HostUnreachable,
        errno::EINTR => Interrupted,
        errno::EINVAL => InvalidInput,
        errno::EISDIR => IsADirectory,
        #[cfg(not(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "loongarch32",
            target_arch = "loongarch64",
            all(target_arch = "xtensa", feature = "openocd-semihosting"),
        )))]
        errno::ELOOP => __FilesystemLoop,
        errno::ENOENT => NotFound,
        errno::ENOMEM => OutOfMemory,
        errno::ENOSPC => StorageFull,
        // errno::ENOSYS => Unsupported,
        errno::EMLINK => TooManyLinks,
        #[cfg(not(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "loongarch32",
            target_arch = "loongarch64",
            all(target_arch = "xtensa", feature = "openocd-semihosting"),
        )))]
        errno::ENAMETOOLONG => InvalidFilename,
        #[cfg(not(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "loongarch32",
            target_arch = "loongarch64",
            all(target_arch = "xtensa", feature = "openocd-semihosting"),
        )))]
        errno::ENETDOWN => NetworkDown,
        #[cfg(not(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "loongarch32",
            target_arch = "loongarch64",
            all(target_arch = "xtensa", feature = "openocd-semihosting"),
        )))]
        errno::ENETUNREACH => NetworkUnreachable,
        #[cfg(not(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "loongarch32",
            target_arch = "loongarch64",
            all(target_arch = "xtensa", feature = "openocd-semihosting"),
        )))]
        errno::ENOTCONN => NotConnected,
        errno::ENOTDIR => NotADirectory,
        // errno::ENOTEMPTY => DirectoryNotEmpty,
        errno::EPIPE => BrokenPipe,
        errno::EROFS => ReadOnlyFilesystem,
        errno::ESPIPE => NotSeekable,
        // errno::ESTALE => StaleNetworkFileHandle,
        #[cfg(not(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "loongarch32",
            target_arch = "loongarch64",
            all(target_arch = "xtensa", feature = "openocd-semihosting"),
        )))]
        errno::ETIMEDOUT => TimedOut,
        #[cfg(not(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "loongarch32",
            target_arch = "loongarch64",
            all(target_arch = "xtensa", feature = "openocd-semihosting"),
        )))]
        errno::ETXTBSY => ExecutableFileBusy,
        errno::EXDEV => CrossesDevices,
        // errno::EINPROGRESS => __InProgress,
        // errno::EOPNOTSUPP => Unsupported,
        errno::EACCES | errno::EPERM => PermissionDenied,

        // These two constants can have the same value on some systems,
        // but different values on others, so we can't use a match
        // clause
        #[cfg(not(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "loongarch32",
            target_arch = "loongarch64",
            all(target_arch = "xtensa", feature = "openocd-semihosting"),
        )))]
        x if x == errno::EAGAIN || x == errno::EWOULDBLOCK => WouldBlock,
        _ => Other,
    }
}
