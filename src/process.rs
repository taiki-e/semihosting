// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A module for working with processes.
//!
//! This module provides [`abort`] and [`exit`] for terminating the current process.
//!
//! See also [`semihosting::sys::arm_compat::sys_system`] for platform-specific
//! semihosting interface to run a system command on the host command-line interpreter.

use crate::sys;

/// Terminates the current process with the specified exit code.
pub fn exit(code: i32) -> ! {
    sys::exit(code);
    loop {}
}

/// Terminates the process in an abnormal fashion.
#[cold]
pub fn abort() -> ! {
    exit(134) // SIGABRT
}
