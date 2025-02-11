// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::{borrow::Borrow, ffi::CStr, fmt, mem::MaybeUninit, num::NonZeroU16, ops, ptr, slice};

pub struct ArrayCString<const CAP: usize> {
    // Length including the NUL byte.
    len: NonZeroU16,

    /// Invariant: CAP is 1..CapType::MAX.
    /// ```
    /// use semihosting::{c, experimental::ArrayCString};
    /// // CAP == 1
    /// let _s = ArrayCString::<1>::try_from(c!(""));
    /// // CAP == u16::MAX
    /// let _s = ArrayCString::<65535>::try_from(c!(""));
    /// ```
    /// ```compile_fail,E0080
    /// use semihosting::{c, experimental::ArrayCString};
    /// // CAP < 1 => const error
    /// let _s = ArrayCString::<0>::try_from(c!(""));
    /// ```
    /// ```compile_fail,E0080
    /// use semihosting::{c, experimental::ArrayCString};
    /// // CAP > u16::MAX => const error
    /// let _s = ArrayCString::<65536>::try_from(c!(""));
    /// ```
    buf: MaybeUninit<[u8; CAP]>,
}

type CapType = u16;
const MAX_CAP: usize = CapType::MAX as usize;

impl<const CAP: usize> ArrayCString<CAP> {
    #[inline]
    fn len_including_nul(&self) -> usize {
        self.len.get() as usize
    }

    #[inline]
    #[must_use]
    pub fn as_bytes_with(&self) -> &[u8] {
        // SAFETY: ArrayCString has a length at least 1
        unsafe {
            slice::from_raw_parts(self.buf.as_ptr().cast::<u8>(), self.len_including_nul() - 1)
        }
    }

    #[inline]
    #[must_use]
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.buf.as_ptr().cast::<u8>(), self.len_including_nul()) }
    }

    #[inline]
    #[must_use]
    pub fn as_c_str(&self) -> &CStr {
        self
    }
}

impl<const CAP: usize> TryFrom<&CStr> for ArrayCString<CAP> {
    type Error = CapacityOverflowError;
    /// # Errors
    ///
    /// - If the `alloc` feature is *not* enabled, this returns an error if `N` is smaller than `s.to_bytes_with_nul().len()`.
    /// - If the `alloc` feature is enabled, this never fails.
    #[inline]
    fn try_from(s: &CStr) -> Result<Self, Self::Error> {
        static_cmp!(usize, CAP > 0);
        static_cmp!(usize, CAP <= MAX_CAP);
        let bytes = s.to_bytes_with_nul();
        let len = bytes.len();
        if CAP >= len {
            let mut buf = MaybeUninit::<[u8; CAP]>::uninit();
            // SAFETY: we've checked `buf` is valid for write of at least `len` bytes.
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), buf.as_mut_ptr().cast::<u8>(), len);
            }
            #[allow(clippy::cast_possible_truncation)] // false positive: we've checked that CAP >= len: https://github.com/rust-lang/rust-clippy/issues/7486
            // SAFETY: bytes returned by to_bytes_with_nul has a length at least 1.
            Ok(Self { buf, len: unsafe { NonZeroU16::new_unchecked(len as CapType) } })
        } else {
            Err(CapacityOverflowError(len))
        }
    }
}

// Turns this `ArrayCString` into an empty string to prevent
// memory-unsafe code from working by accident. Inline
// to prevent LLVM from optimizing it away in debug builds.
impl<const CAP: usize> Drop for ArrayCString<CAP> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: buf has a length at least 1
        unsafe {
            self.buf.as_mut_ptr().cast::<u8>().write(0_u8);
        }
    }
}

impl<const CAP: usize> ops::Deref for ArrayCString<CAP> {
    type Target = CStr;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_bytes_with_nul_unchecked(self.as_bytes_with_nul()) }
    }
}

impl<const CAP: usize> fmt::Debug for ArrayCString<CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<const CAP: usize> Default for ArrayCString<CAP> {
    /// Creates an empty `CString`.
    fn default() -> Self {
        static_cmp!(usize, CAP > 0);
        static_cmp!(usize, CAP <= MAX_CAP);
        let a: &CStr = Default::default();
        // This unwrap will never fail because CAP > 0.
        Self::try_from(a).unwrap()
    }
}

impl<const CAP: usize> AsRef<CStr> for ArrayCString<CAP> {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self
    }
}

impl<const CAP: usize> Borrow<CStr> for ArrayCString<CAP> {
    #[inline]
    fn borrow(&self) -> &CStr {
        self
    }
}

impl<const CAP: usize> ops::Index<ops::RangeFull> for ArrayCString<CAP> {
    type Output = CStr;
    #[inline]
    fn index(&self, _index: ops::RangeFull) -> &Self::Output {
        self
    }
}

#[derive(Debug)]
pub struct CapacityOverflowError(usize);

impl fmt::Display for CapacityOverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "length of provided data is greater than capacity: {}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use core::mem;

    use super::*;

    #[test]
    fn size() {
        assert_eq!(mem::size_of::<Option<ArrayCString<1>>>(), mem::size_of::<ArrayCString<1>>());
        assert_eq!(mem::size_of::<Option<ArrayCString<2>>>(), mem::size_of::<ArrayCString<2>>());
        assert_eq!(mem::size_of::<Option<ArrayCString<3>>>(), mem::size_of::<ArrayCString<3>>());
        assert_eq!(mem::size_of::<Option<ArrayCString<4>>>(), mem::size_of::<ArrayCString<4>>());
        assert_eq!(mem::size_of::<Option<ArrayCString<5>>>(), mem::size_of::<ArrayCString<5>>());
    }

    // TODO
}
