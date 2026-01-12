// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::fmt;

use crate::{fd::AsFd as _, io, sys};

/// Constructs a new handle to the standard input of the current process.
///
/// Unlike [`std::io::stdin`], this function returns the `Result`.
///
/// # Platform-specific behavior
///
/// Currently, this function will always success on MIPS32/MIPS64, On other architectures,
/// this may fail if semihosting is only partially supported.
///
/// Also, we have found that reading from stdin does not work well on MIPS32/MIPS64.
///
/// [`std::io::stdin`]: https://doc.rust-lang.org/std/io/fn.stdin.html
pub fn stdin() -> io::Result<Stdin> {
    sys::stdio::stdin().map(Stdin)
}
/// Constructs a new handle to the standard output of the current process.
///
/// Unlike [`std::io::stdout`], this function returns the `Result`.
///
/// # Platform-specific behavior
///
/// Currently, this function will always success on MIPS32/MIPS64, On other architectures,
/// this may fail if semihosting is only partially supported.
///
/// [`std::io::stdout`]: https://doc.rust-lang.org/std/io/fn.stdout.html
pub fn stdout() -> io::Result<Stdout> {
    sys::stdio::stdout().map(Stdout)
}
/// Constructs a new handle to the standard error of the current process.
///
/// Unlike [`std::io::stderr`], this function returns the `Result`.
///
/// # Platform-specific behavior
///
/// Currently, this function will always success on MIPS32/MIPS64, On other architectures,
/// this may fail if semihosting is only partially supported.
///
/// [`std::io::stderr`]: https://doc.rust-lang.org/std/io/fn.stderr.html
pub fn stderr() -> io::Result<Stderr> {
    sys::stdio::stderr().map(Stderr)
}

/// A handle to the standard input stream of a process.
///
/// Created by the [`io::stdin`] method.
pub struct Stdin(sys::stdio::StdioFd);
/// A handle to the standard output stream of a process.
///
/// Created by the [`io::stdout`] method.
pub struct Stdout(sys::stdio::StdioFd);
/// A handle to the standard error stream of a process.
///
/// Created by the [`io::stderr`] method.
pub struct Stderr(sys::stdio::StdioFd);

impl_as_fd!(Stdin, Stdout, Stderr);
// TODO(stdio): std provides io trait implementations on &Std{in,out,err} as they uses locks.
impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        sys::read(self.as_fd(), buf)
    }
}
impl io::Write for Stdout {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        sys::write(self.as_fd(), bytes)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl io::Write for Stderr {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        sys::write(self.as_fd(), bytes)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl fmt::Debug for Stdin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stdin").finish_non_exhaustive()
    }
}
impl fmt::Debug for Stdout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stdout").finish_non_exhaustive()
    }
}
impl fmt::Debug for Stderr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stderr").finish_non_exhaustive()
    }
}

/// Trait to determine if a descriptor/handle refers to a terminal/tty.
pub trait IsTerminal: crate::sealed::Sealed {
    /// Returns `true` if the descriptor/handle refers to a terminal/tty.
    ///
    /// On platforms where Rust does not know how to detect a terminal yet, this will return
    /// `false`. This will also return `false` if an unexpected error occurred, such as from
    /// passing an invalid file descriptor.
    ///
    /// The following semihosting calls are currently being used:
    ///
    /// | Platform                                           | Semihosting call |
    /// | -------------------------------------------------- | ---------------- |
    /// | AArch64, Arm, RISC-V, Xtensa (openocd-semihosting) | [SYS_ISTTY]      |
    /// | MIPS32, MIPS64                                     | UHI_fstat        |
    ///
    /// [SYS_ISTTY]: https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-istty-0x09
    ///
    /// **Disclaimer:** These semihosting calls might change over time.
    #[doc(alias = "isatty")]
    #[doc(alias = "SYS_ISTTY")] // arm_compat
    fn is_terminal(&self) -> bool;
}
macro_rules! impl_is_terminal {
    ($($t:ty),*$(,)?) => {$(
        impl crate::sealed::Sealed for $t {}
        impl crate::io::IsTerminal for $t {
            #[inline]
            fn is_terminal(&self) -> bool {
                use crate::fd::AsFd as _;
                crate::sys::stdio::is_terminal(self.as_fd())
            }
        }
    )*}
}
impl_is_terminal!(Stdin, Stdout, Stderr);
#[cfg(feature = "fs")]
impl_is_terminal!(crate::fs::File);
