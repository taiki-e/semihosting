// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::ffi::CStr;

use crate::{
    fd::{BorrowedFd, OwnedFd},
    fs, io,
};

pub(crate) struct Metadata {}
impl Metadata {
    pub(crate) fn size(&self) -> u64 {
        // TODO
        0
    }
}
pub(crate) fn metadata(fd: BorrowedFd<'_>) -> io::Result<Metadata> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
pub(crate) fn open(path: &CStr, options: &fs::OpenOptions) -> io::Result<OwnedFd> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
pub(crate) fn seek(fd: BorrowedFd<'_>, pos: io::SeekFrom) -> io::Result<u64> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
pub(crate) fn unlink(path: &CStr) -> io::Result<()> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
pub(crate) fn rename(from: &CStr, to: &CStr) -> io::Result<()> {
    // TODO
    Err(io::ErrorKind::Unsupported.into())
}
