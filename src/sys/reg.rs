// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(single_use_lifetimes)] // false positive

use core::{
    ffi::{c_int, c_void, CStr},
    marker::PhantomData,
};

use crate::{
    fd::{BorrowedFd, RawFd},
    io::RawOsError,
};

/// PARAMETER REGISTER (read-write)
#[repr(transparent)]
pub(crate) struct ParamRegW<'a>(pub(crate) *mut c_void, PhantomData<&'a mut ()>);
impl<'a> ParamRegW<'a> {
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub(crate) fn fd(fd: BorrowedFd<'a>) -> Self {
        Self::raw_fd(fd.as_raw_fd())
    }
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub(crate) fn raw_fd(fd: RawFd) -> Self {
        Self(fd as usize as *mut c_void, PhantomData)
    }
    #[inline]
    pub(crate) fn usize(n: usize) -> Self {
        Self(n as *mut c_void, PhantomData)
    }
    // #[inline]
    // pub(crate) fn isize(n: isize) -> Self {
    //     Self(n as usize as *mut c_void, PhantomData)
    // }
    #[inline]
    pub(crate) fn ref_<T>(r: &'a mut T) -> Self {
        Self((r as *mut T).cast::<c_void>(), PhantomData)
    }
    #[inline]
    pub(crate) fn buf<T>(buf: &'a mut [T]) -> Self {
        Self(buf.as_mut_ptr().cast::<c_void>(), PhantomData)
    }
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv32",
    target_arch = "riscv64",
))]
impl<'a> ParamRegW<'a> {
    #[inline]
    pub(crate) fn block(b: &'a mut [ParamRegW<'_>]) -> Self {
        Self(b.as_mut_ptr().cast::<c_void>(), PhantomData)
    }
}
#[cfg(any(target_arch = "mips", target_arch = "mips64"))]
impl<'a> ParamRegW<'a> {
    #[inline]
    pub(crate) fn ptr<T>(ptr: *mut T) -> Self {
        Self(ptr.cast::<c_void>(), PhantomData)
    }
}

/// PARAMETER REGISTER (read-only)
#[repr(transparent)]
pub(crate) struct ParamRegR<'a>(pub(crate) *const c_void, PhantomData<&'a ()>);
impl<'a> ParamRegR<'a> {
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub(crate) fn fd(fd: BorrowedFd<'a>) -> Self {
        Self::raw_fd(fd.as_raw_fd())
    }
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub(crate) fn raw_fd(fd: RawFd) -> Self {
        Self(fd as usize as *const c_void, PhantomData)
    }
    #[inline]
    pub(crate) fn usize(n: usize) -> Self {
        Self(n as *const c_void, PhantomData)
    }
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub(crate) fn isize(n: isize) -> Self {
        Self(n as usize as *const c_void, PhantomData)
    }
    // #[inline]
    // pub(crate) fn ptr<T>(ptr: *const T) -> Self {
    //     Self(ptr.cast::<c_void>(), PhantomData)
    // }
    #[inline]
    pub(crate) fn buf<T>(buf: &'a [T]) -> Self {
        Self(buf.as_ptr().cast::<c_void>(), PhantomData)
    }
    #[inline]
    pub(crate) fn c_str(s: &'a CStr) -> Self {
        Self(s.as_ptr().cast::<c_void>(), PhantomData)
    }
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv32",
    target_arch = "riscv64",
))]
impl<'a> ParamRegR<'a> {
    #[inline]
    pub(crate) fn block(b: &'a [ParamRegR<'_>]) -> Self {
        Self(b.as_ptr().cast::<c_void>(), PhantomData)
    }
    #[inline]
    pub(crate) fn ref_<T>(r: &'a T) -> Self {
        Self((r as *const T).cast::<c_void>(), PhantomData)
    }
}

/// RETURN REGISTER
#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct RetReg(pub(crate) usize);
impl RetReg {
    #[inline]
    pub(crate) fn usize(self) -> usize {
        self.0
    }
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    #[inline]
    fn isize(self) -> isize {
        self.0 as isize
    }
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    pub(crate) fn int(self) -> c_int {
        self.isize() as c_int
    }
    #[inline]
    pub(crate) fn raw_fd(self) -> Option<RawFd> {
        let fd = self.int();
        if fd == -1 {
            None
        } else {
            debug_assert!(!fd.is_negative(), "{}", fd);
            debug_assert_eq!(fd as isize, self.isize());
            Some(fd)
        }
    }
    #[inline]
    pub(crate) fn errno(self) -> RawOsError {
        let err = self.int();
        debug_assert!(!err.is_negative(), "{}", err);
        debug_assert_eq!(err as isize, self.isize());
        err
    }
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv32",
    target_arch = "riscv64",
))]
impl RetReg {
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    pub(crate) fn u8(self) -> u8 {
        let b = self.usize() as u8;
        debug_assert_eq!(b as usize, self.usize());
        b
    }
}
