// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(missing_docs)]

use core::{
    ffi::{CStr, c_int, c_void},
    marker::PhantomData,
};

#[cfg(semihosting_no_strict_provenance)]
use crate::utils::ptr::PtrExt as _;
use crate::{
    fd::{BorrowedFd, RawFd},
    io::RawOsError,
};

/// PARAMETER REGISTER (read-write)
#[allow(missing_debug_implementations)]
#[repr(transparent)]
pub struct ParamRegW<'a>(pub(crate) *mut c_void, PhantomData<&'a mut ()>);
impl<'a> ParamRegW<'a> {
    #[inline]
    pub fn fd(fd: BorrowedFd<'a>) -> Self {
        Self::raw_fd(fd.as_raw_fd())
    }
    #[inline]
    pub fn raw_fd(fd: RawFd) -> Self {
        Self::signed(fd as isize)
    }
    #[inline]
    pub(crate) fn unsigned(n: usize) -> Self {
        Self(crate::utils::ptr::without_provenance_mut(n), PhantomData)
    }
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub(crate) fn signed(n: isize) -> Self {
        Self::unsigned(n as usize)
    }
    #[inline]
    pub fn usize(n: usize) -> Self {
        // TODO(semver): use without_provenance_mut
        Self(crate::utils::ptr::with_exposed_provenance_mut(n), PhantomData)
    }
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub fn isize(n: isize) -> Self {
        Self::usize(n as usize)
    }
    #[inline]
    pub fn ptr<T>(ptr: *mut T) -> Self {
        Self(ptr.cast::<c_void>(), PhantomData)
    }
    #[inline]
    pub fn ref_<T>(r: &'a mut T) -> Self {
        Self::ptr(r)
    }
    #[inline]
    pub fn buf<T>(buf: &'a mut [T]) -> Self {
        Self::ptr(buf.as_mut_ptr())
    }
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv32",
    target_arch = "riscv64",
    target_arch = "loongarch32",
    target_arch = "loongarch64",
    all(target_arch = "xtensa", feature = "openocd-semihosting"),
))]
impl<'a> ParamRegW<'a> {
    #[inline]
    pub fn block(b: &'a mut [ParamRegW<'_>]) -> Self {
        Self::ptr(b.as_mut_ptr())
    }
    #[inline]
    pub(crate) fn to_ret(&self) -> RetReg {
        RetReg(self.0)
    }
}

/// PARAMETER REGISTER (read-only)
#[allow(missing_debug_implementations)]
#[repr(transparent)]
pub struct ParamRegR<'a>(pub(crate) *const c_void, PhantomData<&'a ()>);
impl<'a> ParamRegR<'a> {
    #[inline]
    pub fn fd(fd: BorrowedFd<'a>) -> Self {
        Self::raw_fd(fd.as_raw_fd())
    }
    #[inline]
    pub fn raw_fd(fd: RawFd) -> Self {
        Self::signed(fd as isize)
    }
    #[inline]
    pub(crate) fn unsigned(n: usize) -> Self {
        Self(crate::utils::ptr::without_provenance_mut(n), PhantomData)
    }
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub(crate) fn signed(n: isize) -> Self {
        Self::unsigned(n as usize)
    }
    #[inline]
    pub fn usize(n: usize) -> Self {
        // TODO(semver): without_provenance_mut
        Self(crate::utils::ptr::with_exposed_provenance(n), PhantomData)
    }
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub fn isize(n: isize) -> Self {
        Self::usize(n as usize)
    }
    #[inline]
    pub fn ptr<T>(ptr: *const T) -> Self {
        Self(ptr.cast::<c_void>(), PhantomData)
    }
    #[inline]
    pub fn ref_<T>(r: &'a T) -> Self {
        Self::ptr(r)
    }
    #[inline]
    pub fn buf<T>(buf: &'a [T]) -> Self {
        Self::ptr(buf.as_ptr())
    }
    #[inline]
    pub fn c_str(s: &'a CStr) -> Self {
        Self::ptr(s.as_ptr())
    }
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv32",
    target_arch = "riscv64",
    target_arch = "loongarch32",
    target_arch = "loongarch64",
    all(target_arch = "xtensa", feature = "openocd-semihosting"),
))]
impl<'a> ParamRegR<'a> {
    #[inline]
    pub fn c_str_len(s: &CStr) -> Self {
        Self::unsigned(s.to_bytes().len())
    }
    #[inline]
    pub fn block(b: &'a [ParamRegR<'_>]) -> Self {
        Self::ptr(b.as_ptr())
    }
}

/// RETURN REGISTER
#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
#[repr(transparent)]
pub struct RetReg(pub(crate) *mut c_void);
impl RetReg {
    #[inline(always)]
    pub(crate) fn unsigned(self) -> usize {
        self.0.addr()
    }
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    #[inline(always)]
    pub(crate) fn signed(self) -> isize {
        self.unsigned() as isize
    }
    #[inline]
    pub fn usize(self) -> usize {
        self.0 as usize
    }
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    #[inline]
    fn isize(self) -> isize {
        self.usize() as isize
    }
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    pub fn int(self) -> c_int {
        self.isize() as c_int
    }
    #[inline]
    pub fn raw_fd(self) -> Option<RawFd> {
        #[allow(clippy::cast_possible_truncation)]
        let fd = self.signed() as RawFd;
        if fd == -1 {
            None
        } else {
            debug_assert!(!fd.is_negative(), "{}", fd);
            debug_assert_eq!(fd as isize, self.signed());
            Some(fd)
        }
    }
    #[inline]
    pub fn errno(self) -> RawOsError {
        #[allow(clippy::cast_possible_truncation)]
        let err = self.signed() as RawOsError;
        debug_assert!(!err.is_negative(), "{}", err);
        debug_assert_eq!(err as isize, self.signed());
        err
    }
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv32",
    target_arch = "riscv64",
    target_arch = "loongarch32",
    target_arch = "loongarch64",
    all(target_arch = "xtensa", feature = "openocd-semihosting"),
))]
impl RetReg {
    #[inline]
    pub fn u8(self) -> u8 {
        #[allow(clippy::cast_possible_truncation)]
        let b = self.unsigned() as u8;
        debug_assert_eq!(b as usize, self.unsigned());
        b
    }
    #[inline(always)]
    pub(crate) fn ptr(self) -> *mut c_void {
        self.0
    }
}
