// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
    io::{self, RawOsError},
    sys::arch::errno,
};

// From https://github.com/rust-lang/rust/blob/1.70.0/library/std/src/sys/unix/mod.rs#L228.
pub(crate) fn decode_error_kind(errno: RawOsError) -> io::ErrorKind {
    #[allow(clippy::enum_glob_use)]
    use io::ErrorKind::*;
    match errno {
        #[cfg(not(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        )))] // TODO
        errno::E2BIG => __ArgumentListTooLong, // unstable
        // errno::EADDRINUSE => AddrInUse,
        // errno::EADDRNOTAVAIL => AddrNotAvailable,
        errno::EBUSY => __ResourceBusy, // unstable
        // errno::ECONNABORTED => ConnectionAborted,
        // errno::ECONNREFUSED => ConnectionRefused,
        #[cfg(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        ))] // TODO
        errno::ECONNRESET => ConnectionReset,
        // errno::EDEADLK => Deadlock, // unstable
        // errno::EDQUOT => FilesystemQuotaExceeded, // unstable
        errno::EEXIST => AlreadyExists,
        errno::EFBIG => __FileTooLarge, // unstable
        // errno::EHOSTUNREACH => HostUnreachable, // unstable
        errno::EINTR => Interrupted,
        errno::EINVAL => InvalidInput,
        errno::EISDIR => __IsADirectory, // unstable
        // errno::ELOOP => FilesystemLoop, // unstable
        errno::ENOENT => NotFound,
        errno::ENOMEM => OutOfMemory,
        errno::ENOSPC => __StorageFull, // unstable
        // errno::ENOSYS => Unsupported,
        errno::EMLINK => __TooManyLinks, // unstable
        // errno::ENAMETOOLONG => InvalidFilename, // unstable
        // errno::ENETDOWN => NetworkDown, // unstable
        // errno::ENETUNREACH => NetworkUnreachable, // unstable
        #[cfg(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        ))] // TODO
        errno::ENOTCONN => NotConnected,
        errno::ENOTDIR => __NotADirectory, // unstable
        // errno::ENOTEMPTY => DirectoryNotEmpty, // unstable
        errno::EPIPE => BrokenPipe,
        errno::EROFS => __ReadOnlyFilesystem, // unstable
        errno::ESPIPE => __NotSeekable,       // unstable
        // errno::ESTALE => StaleNetworkFileHandle, // unstable
        #[cfg(any(
            target_arch = "mips",
            target_arch = "mips32r6",
            target_arch = "mips64",
            target_arch = "mips64r6",
        ))] // TODO
        errno::ETIMEDOUT => TimedOut,
        // errno::ETXTBSY => ExecutableFileBusy, // unstable
        errno::EXDEV => __CrossesDevices, // unstable
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
