// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::fmt;

use crate::sys;

/// A specialized [`Result`] type for I/O operations.
///
/// See [`std::io::Result` documentation][std] for details.
///
/// [std]: https://doc.rust-lang.org/std/io/type.Result.html
pub type Result<T> = core::result::Result<T, Error>;

/// The type of raw OS error codes returned by [`Error::raw_os_error`].
///
/// See [`std::io::RawOsError` documentation][std] for details.
///
/// [std]: https://doc.rust-lang.org/nightly/std/io/type.RawOsError.html
pub type RawOsError = i32;

/// The error type for I/O operations.
///
/// See [`std::io::Error` documentation][std] for details.
///
/// [std]: https://doc.rust-lang.org/std/io/struct.Error.html
pub struct Error {
    repr: Repr,
}

enum Repr {
    Os(RawOsError),
    Simple(ErrorKind),
    SimpleMessage(&'static SimpleMessage),
}

pub(crate) struct SimpleMessage {
    kind: ErrorKind,
    message: &'static str,
}

impl SimpleMessage {
    pub(crate) const fn new(kind: ErrorKind, message: &'static str) -> Self {
        Self { kind, message }
    }
}

/// Create and return an `io::Error` for a given `ErrorKind` and constant
/// message. This doesn't allocate.
macro_rules! const_io_error {
    ($kind:expr, $message:expr $(,)?) => {
        $crate::io::error::Error::from_static_message({
            const MESSAGE_DATA: $crate::io::error::SimpleMessage =
                $crate::io::error::SimpleMessage::new($kind, $message);
            &MESSAGE_DATA
        })
    };
}

/// A list specifying general categories of I/O error.
///
/// See [`std::io::ErrorKind` documentation][std] for details.
///
/// [std]: https://doc.rust-lang.org/std/io/enum.ErrorKind.html
#[allow(missing_docs)] // TODO
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    #[doc(hidden)]
    __HostUnreachable, // unstable
    #[doc(hidden)]
    __NetworkUnreachable, // unstable
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    #[doc(hidden)]
    __NetworkDown, // unstable
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    #[doc(hidden)]
    __NotADirectory, // unstable
    #[doc(hidden)]
    __IsADirectory, // unstable
    #[doc(hidden)]
    __DirectoryNotEmpty, // unstable
    #[doc(hidden)]
    __ReadOnlyFilesystem, // unstable
    #[doc(hidden)]
    __FilesystemLoop, // unstable
    #[doc(hidden)]
    __StaleNetworkFileHandle, // unstable
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    #[doc(hidden)]
    __StorageFull, // unstable
    #[doc(hidden)]
    __NotSeekable, // unstable
    #[doc(hidden)]
    __FilesystemQuotaExceeded, // unstable
    #[doc(hidden)]
    __FileTooLarge, // unstable
    #[doc(hidden)]
    __ResourceBusy, // unstable
    #[doc(hidden)]
    __ExecutableFileBusy, // unstable
    #[doc(hidden)]
    __Deadlock, // unstable
    #[doc(hidden)]
    __CrossesDevices, // unstable
    #[doc(hidden)]
    __TooManyLinks, // unstable
    #[doc(hidden)]
    __InvalidFilename, // unstable
    #[doc(hidden)]
    __ArgumentListTooLong, // unstable
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    Other,
    // Uncategorized, // unstable, private api
}

impl ErrorKind {
    pub(crate) fn as_str(self) -> &'static str {
        #[allow(clippy::enum_glob_use)]
        use ErrorKind::*;
        match self {
            AddrInUse => "address in use",
            AddrNotAvailable => "address not available",
            AlreadyExists => "entity already exists",
            __ArgumentListTooLong => "argument list too long",
            BrokenPipe => "broken pipe",
            ConnectionAborted => "connection aborted",
            ConnectionRefused => "connection refused",
            ConnectionReset => "connection reset",
            __CrossesDevices => "cross-device link or rename",
            __Deadlock => "deadlock",
            __DirectoryNotEmpty => "directory not empty",
            __ExecutableFileBusy => "executable file busy",
            __FileTooLarge => "file too large",
            __FilesystemLoop => "filesystem loop or indirection limit (e.g. symlink loop)",
            __FilesystemQuotaExceeded => "filesystem quota exceeded",
            __HostUnreachable => "host unreachable",
            Interrupted => "operation interrupted",
            InvalidData => "invalid data",
            __InvalidFilename => "invalid filename",
            InvalidInput => "invalid input parameter",
            __IsADirectory => "is a directory",
            __NetworkDown => "network down",
            __NetworkUnreachable => "network unreachable",
            __NotADirectory => "not a directory",
            NotConnected => "not connected",
            NotFound => "entity not found",
            __NotSeekable => "seek on unseekable file",
            Other => "other error",
            OutOfMemory => "out of memory",
            PermissionDenied => "permission denied",
            __ReadOnlyFilesystem => "read-only filesystem or storage medium",
            __ResourceBusy => "resource busy",
            __StaleNetworkFileHandle => "stale network file handle",
            __StorageFull => "no storage space",
            TimedOut => "timed out",
            __TooManyLinks => "too many links",
            // Uncategorized => "uncategorized error",
            UnexpectedEof => "unexpected end of file",
            Unsupported => "unsupported",
            WouldBlock => "operation would block",
            WriteZero => "write zero",
        }
    }
}

impl fmt::Display for ErrorKind {
    /// Shows a human-readable description of the `ErrorKind`.
    ///
    /// This is similar to `impl Display for Error`, but doesn't require first converting to Error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Intended for use for errors not exposed to the user, where allocating onto
/// the heap (for normal construction via Error::new) is too costly.
impl From<ErrorKind> for Error {
    /// Converts an [`ErrorKind`] into an [`Error`].
    ///
    /// This conversion creates a new error with a simple representation of error kind.
    #[inline]
    fn from(kind: ErrorKind) -> Self {
        Self { repr: Repr::Simple(kind) }
    }
}

impl Error {
    #[inline]
    pub(crate) const fn from_static_message(msg: &'static SimpleMessage) -> Error {
        Self { repr: Repr::SimpleMessage(msg) }
    }
    /// Creates a new instance of an `Error` from a particular OS error code.
    #[must_use]
    #[inline]
    pub fn from_raw_os_error(os: RawOsError) -> Self {
        Self { repr: Repr::Os(os) }
    }
    /// Returns the OS error that this error represents (if any).
    #[must_use]
    #[inline]
    pub fn raw_os_error(&self) -> Option<RawOsError> {
        match self.repr {
            Repr::Os(code) => Some(code),
            Repr::Simple(..) | Repr::SimpleMessage(..) => None,
        }
    }
    /// Returns the corresponding [`ErrorKind`] for this error.
    #[must_use]
    #[inline]
    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            Repr::Os(code) => sys::decode_error_kind(code),
            Repr::Simple(kind) => kind,
            Repr::SimpleMessage(msg) => msg.kind,
        }
    }
}

impl fmt::Debug for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Os(code) => f
                .debug_struct("Os")
                .field("code", &code)
                .field("kind", &sys::decode_error_kind(*code))
                // TODO
                // .field("message", &sys::os::error_string(code))
                .finish(),
            Self::Simple(kind) => f.debug_tuple("Kind").field(&kind).finish(),
            Self::SimpleMessage(msg) => f
                .debug_struct("Error")
                .field("kind", &msg.kind)
                .field("message", &msg.message)
                .finish(),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.repr, f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repr {
            Repr::Os(code) => {
                // TODO
                // let detail = sys::os::error_string(code);
                // write!(f, "{detail} (os error {code})")
                let detail = sys::decode_error_kind(code);
                write!(f, "{detail} (os error {code})")
            }
            Repr::Simple(kind) => f.write_str(kind.as_str()),
            Repr::SimpleMessage(msg) => msg.message.fmt(f),
        }
    }
}

#[cfg(not(semihosting_no_error_in_core))]
impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self.repr {
            Repr::Os(..) | Repr::Simple(..) | Repr::SimpleMessage(..) => None,
            // Repr::Custom(c) => c.error.source(),
        }
    }
}
