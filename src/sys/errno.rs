// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::ffi;

use crate::{
    io::{self, RawOsError},
    sys::arch::errno,
};

#[inline]
pub(crate) fn is_interrupted(errno: RawOsError) -> bool {
    errno as ffi::c_int == errno::EINTR
}

// Adapted from https://github.com/rust-lang/rust/blob/1.92.0/library/std/src/sys/pal/unix/mod.rs#L235.
pub(crate) fn decode_error_kind(errno: RawOsError) -> io::ErrorKind {
    #[allow(clippy::enum_glob_use)]
    use io::ErrorKind::*;
    match errno as ffi::c_int {
        #[cfg(not(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        )))] // TODO
        errno::E2BIG => ArgumentListTooLong,
        // errno::EADDRINUSE => AddrInUse,
        // errno::EADDRNOTAVAIL => AddrNotAvailable,
        errno::EBUSY => ResourceBusy,
        // errno::ECONNABORTED => ConnectionAborted,
        // errno::ECONNREFUSED => ConnectionRefused,
        #[cfg(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        ))] // TODO
        errno::ECONNRESET => ConnectionReset,
        // errno::EDEADLK => Deadlock,
        // errno::EDQUOT => QuotaExceeded,
        errno::EEXIST => AlreadyExists,
        errno::EFBIG => FileTooLarge,
        // errno::EHOSTUNREACH => HostUnreachable,
        errno::EINTR => Interrupted,
        errno::EINVAL => InvalidInput,
        errno::EISDIR => IsADirectory,
        #[cfg(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        ))] // TODO
        errno::ELOOP => __FilesystemLoop,
        errno::ENOENT => NotFound,
        errno::ENOMEM => OutOfMemory,
        errno::ENOSPC => StorageFull,
        // errno::ENOSYS => Unsupported,
        errno::EMLINK => TooManyLinks,
        #[cfg(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        ))] // TODO
        errno::ENAMETOOLONG => InvalidFilename,
        #[cfg(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        ))] // TODO
        errno::ENETDOWN => NetworkDown,
        #[cfg(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        ))] // TODO
        errno::ENETUNREACH => NetworkUnreachable,
        #[cfg(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        ))] // TODO
        errno::ENOTCONN => NotConnected,
        errno::ENOTDIR => NotADirectory,
        // errno::ENOTEMPTY => DirectoryNotEmpty,
        errno::EPIPE => BrokenPipe,
        errno::EROFS => ReadOnlyFilesystem,
        errno::ESPIPE => NotSeekable,
        // errno::ESTALE => StaleNetworkFileHandle,
        #[cfg(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        ))] // TODO
        errno::ETIMEDOUT => TimedOut,
        #[cfg(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        ))] // TODO
        errno::ETXTBSY => ExecutableFileBusy,
        errno::EXDEV => CrossesDevices,
        // errno::EINPROGRESS => __InProgress,
        // errno::EOPNOTSUPP => Unsupported,
        errno::EACCES | errno::EPERM => PermissionDenied,

        // These two constants can have the same value on some systems,
        // but different values on others, so we can't use a match
        // clause
        #[cfg(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        ))] // TODO
        x if x == errno::EAGAIN || x == errno::EWOULDBLOCK => WouldBlock,
        _ => Other,
    }
}
