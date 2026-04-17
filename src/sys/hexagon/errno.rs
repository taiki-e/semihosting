// SPDX-License-Identifier: Apache-2.0 OR MIT

// Refs: https://github.com/quic/qemu/blob/3eb022b82002e58cf0636b01e4ee2b3a7878d822/target/hexagon/hexswi.c#L98

// TODO(sys): should we expose this as public API of `sys` module?

#![allow(dead_code)]

use core::ffi::c_int;

pub(crate) const EPERM: c_int = 1;
pub(crate) const ENOENT: c_int = 2;
pub(crate) const EINTR: c_int = 4;
pub(crate) const EIO: c_int = 5;
pub(crate) const ENXIO: c_int = 6;
pub(crate) const EBADF: c_int = 9;
pub(crate) const EAGAIN: c_int = 11;
pub(crate) const ENOMEM: c_int = 12;
pub(crate) const EACCES: c_int = 13;
pub(crate) const EFAULT: c_int = 14;
pub(crate) const EBUSY: c_int = 16;
pub(crate) const EEXIST: c_int = 17;
pub(crate) const EXDEV: c_int = 18;
pub(crate) const ENODEV: c_int = 19;
pub(crate) const ENOTDIR: c_int = 20;
pub(crate) const EISDIR: c_int = 21;
pub(crate) const EINVAL: c_int = 22;
pub(crate) const ENFILE: c_int = 23;
pub(crate) const EMFILE: c_int = 24;
pub(crate) const ENOTTY: c_int = 25;
pub(crate) const ETXTBSY: c_int = 26;
pub(crate) const EFBIG: c_int = 27;
pub(crate) const ENOSPC: c_int = 28;
pub(crate) const ESPIPE: c_int = 29;
pub(crate) const EROFS: c_int = 30;
pub(crate) const EMLINK: c_int = 31;
pub(crate) const EPIPE: c_int = 32;
pub(crate) const ERANGE: c_int = 34;
pub(crate) const ENAMETOOLONG: c_int = 36;
pub(crate) const ENOSYS: c_int = 38;
pub(crate) const ELOOP: c_int = 40;
pub(crate) const EOVERFLOW: c_int = 75;
