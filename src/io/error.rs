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
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    HostUnreachable,
    NetworkUnreachable,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    NetworkDown,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    ReadOnlyFilesystem,
    #[doc(hidden)]
    __FilesystemLoop, // unstable https://github.com/rust-lang/rust/issues/130188
    StaleNetworkFileHandle,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    StorageFull,
    NotSeekable,
    QuotaExceeded,
    FileTooLarge,
    ResourceBusy,
    ExecutableFileBusy,
    Deadlock,
    CrossesDevices,
    TooManyLinks,
    #[doc(hidden)]
    __InvalidFilename, // unstable https://github.com/rust-lang/rust/issues/130192 / https://github.com/rust-lang/rust/pull/134076
    ArgumentListTooLong,
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    #[doc(hidden)]
    __InProgress, // unstable https://github.com/rust-lang/rust/issues/130840
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
            ArgumentListTooLong => "argument list too long",
            BrokenPipe => "broken pipe",
            ConnectionAborted => "connection aborted",
            ConnectionRefused => "connection refused",
            ConnectionReset => "connection reset",
            CrossesDevices => "cross-device link or rename",
            Deadlock => "deadlock",
            DirectoryNotEmpty => "directory not empty",
            ExecutableFileBusy => "executable file busy",
            FileTooLarge => "file too large",
            __FilesystemLoop => "filesystem loop or indirection limit (e.g. symlink loop)",
            HostUnreachable => "host unreachable",
            Interrupted => "operation interrupted",
            __InProgress => "in progress",
            InvalidData => "invalid data",
            __InvalidFilename => "invalid filename",
            InvalidInput => "invalid input parameter",
            IsADirectory => "is a directory",
            NetworkDown => "network down",
            NetworkUnreachable => "network unreachable",
            NotADirectory => "not a directory",
            NotConnected => "not connected",
            NotFound => "entity not found",
            NotSeekable => "seek on unseekable file",
            Other => "other error",
            OutOfMemory => "out of memory",
            PermissionDenied => "permission denied",
            QuotaExceeded => "quota exceeded",
            ReadOnlyFilesystem => "read-only filesystem or storage medium",
            ResourceBusy => "resource busy",
            StaleNetworkFileHandle => "stale network file handle",
            StorageFull => "no storage space",
            TimedOut => "timed out",
            TooManyLinks => "too many links",
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
    #[inline]
    #[must_use]
    pub fn from_raw_os_error(os: RawOsError) -> Self {
        Self { repr: Repr::Os(os) }
    }
    /// Returns the OS error that this error represents (if any).
    #[inline]
    #[must_use]
    pub fn raw_os_error(&self) -> Option<RawOsError> {
        match self.repr {
            Repr::Os(code) => Some(code),
            Repr::Simple(..) | Repr::SimpleMessage(..) => None,
        }
    }
    /// Returns the corresponding [`ErrorKind`] for this error.
    #[inline]
    #[must_use]
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
