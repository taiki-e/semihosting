// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Inspection and manipulation of the process's environment.
//!
//! This module is a subset of the [`std::env`] module, with [some differences](https://github.com/taiki-e/semihosting/issues/1).
//!
//! [`std::env`]: https://doc.rust-lang.org/std/env/index.html

#![allow(clippy::undocumented_unsafe_blocks)] // TODO

use core::{fmt, mem::MaybeUninit, str};

use crate::{io, sys::env as sys};

/// An iterator over the arguments of a process, yielding a `Result<&str>` value for
/// each argument.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Args<const BUF_SIZE: usize>(sys::ArgsBytes<BUF_SIZE>);

/// Returns the arguments that this program was started with.
pub fn args<const BUF_SIZE: usize>() -> io::Result<Args<BUF_SIZE>> {
    sys::args_bytes().map(Args)
}

#[allow(clippy::copy_iterator)] // TODO(args)
impl<'a, const BUF_SIZE: usize> Iterator for &'a Args<BUF_SIZE> {
    type Item = Result<&'a str, str::Utf8Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let arg = sys::next(&self.0)?;
        Some(str::from_utf8(arg))
    }
}

impl<const BUF_SIZE: usize> fmt::Debug for Args<BUF_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Args").finish_non_exhaustive()
    }
}

/// An iterator over the arguments of a process, reading into a buffer provided
/// by the caller, yielding a `Result<&str>` value for each argument.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ArgsIn<'a>(sys::ArgsBytesRef<'a>);

/// Returns the arguments that this program was started with, reading them into
/// `buf`.
///
/// The caller chooses the size of the buffer and where it lives, and the buffer
/// is never copied.
///
/// [`args`] owns its buffer instead. Returning it by value copies it out of the
/// frame the semihosting call filled, so the peak stack use is twice the buffer
/// size. Prefer this function where that matters: a buffer in a `static` keeps
/// the arguments off the stack entirely.
///
/// # Examples
///
/// ```no_run
/// use core::mem::MaybeUninit;
///
/// let mut buf = [MaybeUninit::uninit(); 64];
/// let args = semihosting::experimental::env::args_in(&mut buf)?;
/// for arg in &args {
///     let _: &str = arg?;
/// }
/// # Ok::<(), semihosting::io::Error>(())
/// ```
pub fn args_in(buf: &mut [MaybeUninit<u8>]) -> io::Result<ArgsIn<'_>> {
    sys::args_bytes_in(buf).map(ArgsIn)
}

#[allow(clippy::copy_iterator)] // TODO(args)
impl<'a> Iterator for &ArgsIn<'a> {
    type Item = Result<&'a str, str::Utf8Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let arg = sys::next_ref(&self.0)?;
        Some(str::from_utf8(arg))
    }
}

impl fmt::Debug for ArgsIn<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArgsIn").finish_non_exhaustive()
    }
}
