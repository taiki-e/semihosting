// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::ffi::CStr;

use crate::{
    fd::{BorrowedFd, OwnedFd},
    fs, io,
};

pub(crate) struct Metadata {
    _private: (),
}

impl Metadata {
    #[inline]
    #[allow(clippy::unused_self)]
    pub(crate) fn size(&self) -> u64 {
        0
    }
}

pub(crate) fn metadata(_fd: BorrowedFd<'_>) -> io::Result<Metadata> {
    unimplemented!("semihosting is not available on this target")
}

pub(crate) fn open(_path: &CStr, _options: &fs::OpenOptions) -> io::Result<OwnedFd> {
    unimplemented!("semihosting is not available on this target")
}

pub(crate) fn seek(_fd: BorrowedFd<'_>, _pos: io::SeekFrom) -> io::Result<u64> {
    unimplemented!("semihosting is not available on this target")
}

pub(crate) fn unlink(_path: &CStr) -> io::Result<()> {
    unimplemented!("semihosting is not available on this target")
}

pub(crate) fn rename(_from: &CStr, _to: &CStr) -> io::Result<()> {
    unimplemented!("semihosting is not available on this target")
}
