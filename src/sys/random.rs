// SPDX-License-Identifier: Apache-2.0 OR MIT

// Refs: https://github.com/rust-lang/rust/blob/1.92.0/library/std/src/sys/random/unix_legacy.rs

use core::mem::{self, MaybeUninit};

use crate::{
    fd::{BorrowedFd, OwnedFd},
    fs, io, sys,
    utils::slice_assume_init_mut,
};

cfg_sel!({
    #[cfg(any(target_has_atomic = "32", feature = "portable-atomic"))]
    {
        use self::once::OnceOwnedFd;
        static DEVICE: OnceOwnedFd = OnceOwnedFd::none();

        pub(crate) fn fill_bytes(bytes: &mut [MaybeUninit<u8>]) -> io::Result<&mut [u8]> {
            #[cold]
            fn init() -> io::Result<OwnedFd> {
                Ok(fs::File::open(c!("/dev/urandom"))?.into())
            }
            let fd = DEVICE.get_or_try_init(init)?;
            read_exact_uninit(fd, bytes)
        }
    }
    #[cfg(else)]
    {
        use crate::fd::AsFd as _;

        pub(crate) fn fill_bytes(bytes: &mut [MaybeUninit<u8>]) -> io::Result<&mut [u8]> {
            let fd: OwnedFd = fs::File::open(c!("/dev/urandom"))?.into();
            read_exact_uninit(fd.as_fd(), bytes)
        }
    }
});

fn read_exact_uninit<'a>(
    fd: BorrowedFd<'_>,
    buf: &'a mut [MaybeUninit<u8>],
) -> io::Result<&'a mut [u8]> {
    fn inner(fd: BorrowedFd<'_>, mut buf: &mut [MaybeUninit<u8>]) -> io::Result<()> {
        if buf.is_empty() {
            return Ok(());
        }
        loop {
            match sys::read_uninit(fd, mem::take(&mut buf)) {
                Ok((&mut [], _) | (_, &mut [])) => break,
                Ok((_, rest)) => {
                    buf = rest;
                }
                Err(ref e) if e.is_interrupted() => {}
                Err(e) => return Err(e),
            }
        }
        if buf.is_empty() { Ok(()) } else { Err(io::Error::READ_EXACT_EOF) }
    }
    inner(fd, buf)?;
    // SAFETY: we've filled all bytes
    Ok(unsafe { slice_assume_init_mut(buf) })
}

#[cfg(any(target_has_atomic = "32", feature = "portable-atomic"))]
mod once {
    use crate::{
        atomic::{AtomicI32, Ordering},
        fd::{BorrowedFd, OwnedFd},
        io,
    };
    const INIT: i32 = -1;
    #[repr(transparent)]
    pub(super) struct OnceOwnedFd(AtomicI32);
    impl OnceOwnedFd {
        pub(super) const fn none() -> Self {
            Self(AtomicI32::new(INIT))
        }
        #[inline]
        fn get(&self) -> Option<BorrowedFd<'_>> {
            let fd = self.0.load(Ordering::Acquire);
            // SAFETY: we set non -1 value only from OwnedFd and close it only from Drop.
            if fd == INIT { None } else { Some(unsafe { BorrowedFd::borrow_raw(fd) }) }
        }
        #[inline]
        pub(super) fn get_or_try_init(
            &self,
            f: impl FnOnce() -> io::Result<OwnedFd>,
        ) -> io::Result<BorrowedFd<'_>> {
            if let Some(fd) = self.get() {
                return Ok(fd);
            }
            self.try_init(f)
        }
        #[cold]
        fn try_init(&self, f: impl FnOnce() -> io::Result<OwnedFd>) -> io::Result<BorrowedFd<'_>> {
            let fd = f()?;
            if let Some(fd) = self.get() {
                return Ok(fd);
            }
            let fd = fd.into_raw_fd();
            match self.0.compare_exchange(INIT, fd, Ordering::Release, Ordering::Acquire) {
                // SAFETY: we set non -1 value only from OwnedFd and close it only from Drop.
                Ok(_) => Ok(unsafe { BorrowedFd::borrow_raw(fd) }),
                Err(new_fd) => {
                    // SAFETY: fd is from OwnedFd and will never referred from others since CAS failed.
                    drop(unsafe { OwnedFd::from_raw_fd(fd) });
                    // SAFETY: we set non -1 value only from OwnedFd and close it only from Drop.
                    Ok(unsafe { BorrowedFd::borrow_raw(new_fd) })
                }
            }
        }
    }
    impl Drop for OnceOwnedFd {
        fn drop(&mut self) {
            let fd = *self.0.get_mut();
            if fd != INIT {
                // SAFETY: we set non -1 value only from OwnedFd and close it only from Drop.
                drop(unsafe { OwnedFd::from_raw_fd(fd) });
            }
        }
    }
}
