// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::ffi::CStr;

use super::{
    LseekFlag::SEEK_SET, O_APPEND, O_CREAT, O_EXCL, O_RDONLY, O_RDWR, O_TRUNC, O_WRONLY,
    hosted_fstat, hosted_lseek, hosted_open,
};
pub(crate) use super::{
    hosted_fstat as metadata, hosted_rename as rename, hosted_unlink as unlink, stat as Metadata,
};
use crate::{
    fd::{BorrowedFd, OwnedFd},
    fs, io,
    sys::errno::einval,
};

impl Metadata {
    pub(crate) fn size(&self) -> u64 {
        self.st_size
    }
}

#[allow(clippy::cast_possible_wrap)]
pub(crate) fn open(path: &CStr, options: &fs::OpenOptions) -> io::Result<OwnedFd> {
    match (options.write, options.append) {
        (true, false) => {}
        (false, false) => {
            if options.truncate || options.create {
                return Err(einval());
            }
        }
        (_, true) => {
            if options.truncate {
                return Err(einval());
            }
        }
    }
    // TODO
    let access_mode = match (options.read, options.write, options.append) {
        (true, false, false) => O_RDONLY,
        (false, true, false) => O_WRONLY,
        (true, true, false) => O_RDWR,
        (false, _, true) => O_WRONLY | O_APPEND,
        (true, _, true) => O_RDWR | O_APPEND,
        (false, false, false) => return Err(einval()),
    };
    let creation_mode = match (options.create, options.truncate, options.create_new) {
        (false, false, false) => 0,
        (true, false, false) => O_CREAT,
        (false, true, false) => O_TRUNC,
        (true, true, false) => O_CREAT | O_TRUNC,
        (_, _, true) => O_CREAT | O_EXCL,
    };
    hosted_open(path, access_mode | creation_mode, options.mode as u32)
}

#[allow(clippy::cast_possible_wrap)]
pub(crate) fn seek(fd: BorrowedFd<'_>, pos: io::SeekFrom) -> io::Result<u64> {
    let (whence, offset) = match pos {
        io::SeekFrom::Start(pos) => (SEEK_SET, pos as i64),
        io::SeekFrom::End(offset) => {
            // TODO: this is mips-specific?
            // (SEEK_END, offset) doesn't reject invalid offset
            let len = hosted_fstat(fd)?.size();
            let pos = (len as i64).saturating_add(offset);
            if pos.is_negative() {
                return Err(einval());
            }
            (SEEK_SET, pos)
        } // io::SeekFrom::Current(offset) => (SEEK_CUR, offset),
    };
    Ok(unsafe { hosted_lseek(fd, offset, whence)? as u64 })
}
