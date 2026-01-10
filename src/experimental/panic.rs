// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Panic support.
//!
//! This module is a subset of the [`std::panic`] module.
//!
//! [`std::panic`]: https://doc.rust-lang.org/std/panic/index.html

use alloc::boxed::Box;
use core::any::Any;

use crate::atomic::{AtomicUsize, Ordering};

pub(crate) static PANICKED: AtomicUsize = AtomicUsize::new(0);

/// Invokes a closure, capturing the cause of an unwinding panic if one occurs.
pub fn catch_unwind<F: FnOnce() -> R, R>(f: F) -> Result<R, Box<dyn Any + Send>> {
    let res = unwinding::panic::catch_unwind(f);
    if res.is_err() {
        let panicked = PANICKED.fetch_sub(1, Ordering::Release);
        debug_assert!(panicked != 0);
    }
    res
}
