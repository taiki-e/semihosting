// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::ffi::CStr;

use crate::{
    fd::{BorrowedFd, OwnedFd},
    io::{self, Error, Result},
};

use super::{errno, sys_flen, sys_open, sys_seek, OpenMode};
pub(crate) use super::{sys_remove as unlink, sys_rename as rename};

pub(crate) struct Metadata {
    size: u64,
}

impl Metadata {
    pub(crate) fn size(&self) -> u64 {
        self.size
    }
}

pub(crate) fn metadata(fd: BorrowedFd<'_>) -> Result<Metadata> {
    Ok(Metadata { size: sys_flen(fd)? as u64 })
}

pub(crate) fn open(path: &CStr, options: &crate::fs::OpenOptions) -> Result<OwnedFd> {
    match (options.write, options.append) {
        (true, false) => {}
        (false, false) => {
            if options.truncate || options.create {
                return Err(Error::from_raw_os_error(errno::EINVAL));
            }
        }
        (_, true) => {
            if options.truncate {
                return Err(Error::from_raw_os_error(errno::EINVAL));
            }
        }
    }
    // Refs: https://github.com/openocd-org/openocd/blob/master/src/target/semihosting_common.c
    let mode = match (options.read, options.write, options.append, options.create, options.truncate)
    {
        (true, false, false, false, false) => OpenMode::RDONLY,
        (true, true, false, false, false) => OpenMode::RDWR,
        (false, true, false, true, true) => OpenMode::WRONLY_TRUNC,
        (true, true, false, true, true) => OpenMode::RDWR_TRUNC,
        (false, true, true, true, false) => OpenMode::WRONLY_APPEND,
        (true, true, true, true, false) => OpenMode::RDWR_APPEND,
        _ => return Err(Error::from_raw_os_error(errno::EINVAL)),
    };
    sys_open(path, mode)
}

#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub(crate) fn seek(fd: BorrowedFd<'_>, pos: io::SeekFrom) -> Result<u64> {
    let abs_pos = match pos {
        io::SeekFrom::Start(pos) => pos,
        io::SeekFrom::End(offset) => {
            let len = sys_flen(fd)? as u64;
            let pos = (len as i64).saturating_add(offset);
            if pos.is_negative() {
                return Err(Error::from_raw_os_error(errno::EINVAL));
            }
            pos as u64
        } // io::SeekFrom::Current(_offset) => todo!(),
    };
    // sys_seek may succeed without this guard, but make the behavior consistent with other platforms.
    let abs_pos = isize::try_from(abs_pos).map_err(|_| Error::from_raw_os_error(errno::EINVAL))?;
    unsafe {
        sys_seek(fd, abs_pos as usize)?;
    }
    Ok(abs_pos as u64)
}
