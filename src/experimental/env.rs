// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Inspection and manipulation of the process's environment.
//!
//! This module is a subset of the [`std::env`] module, with [some differences](https://github.com/taiki-e/semihosting/issues/1).
//!
//! [`std::env`]: https://doc.rust-lang.org/std/env/index.html

#![allow(clippy::undocumented_unsafe_blocks)] // TODO

use core::{fmt, str};

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
