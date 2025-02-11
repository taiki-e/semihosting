// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(feature = "alloc")]
use alloc::ffi::CString;
#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};
#[cfg(feature = "alloc")]
use core::num::NonZeroU8;
use core::{borrow::Borrow, ffi::CStr, fmt, mem::MaybeUninit, ops, ptr, slice};

pub struct SmallCString<const N: usize> {
    repr: Repr<N>,
}

/// ```compile_fail,E0080
/// use semihosting::{ffi::*, c};
/// let _s = SmallCString::<0>::try_from(c!(""));
/// ```
enum Repr<const N: usize> {
    // Length including the NUL byte
    Inline(MaybeUninit<[u8; N]>, usize),
    #[cfg(feature = "alloc")]
    Heap(CString),
}

impl<const N: usize> SmallCString<N> {
    #[inline]
    #[must_use]
    pub fn as_bytes_with(&self) -> &[u8] {
        match &self.repr {
            // SAFETY: SmallCString has a length at least 1
            Repr::Inline(b, len) => unsafe {
                slice::from_raw_parts(b.as_ptr().cast::<u8>(), *len - 1)
            },
            #[cfg(feature = "alloc")]
            Repr::Heap(s) => s.as_bytes(),
        }
    }

    #[inline]
    #[must_use]
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        match &self.repr {
            Repr::Inline(b, len) => unsafe { slice::from_raw_parts(b.as_ptr().cast::<u8>(), *len) },
            #[cfg(feature = "alloc")]
            Repr::Heap(s) => s.as_bytes_with_nul(),
        }
    }

    #[inline]
    #[must_use]
    pub fn as_c_str(&self) -> &CStr {
        self
    }
}

impl<const N: usize> TryFrom<&CStr> for SmallCString<N> {
    type Error = CapacityOverflowError;
    /// # Errors
    ///
    /// - If the `alloc` feature is *not* enabled, this returns an error if `N` is smaller than `s.to_bytes_with_nul().len()`.
    /// - If the `alloc` feature is enabled, this never fails.
    #[inline]
    fn try_from(s: &CStr) -> Result<Self, Self::Error> {
        static_cmp!(usize, N > 0);
        let bytes = s.to_bytes_with_nul();
        let len = bytes.len();
        if N >= len {
            let mut buf = MaybeUninit::<[u8; N]>::uninit();
            let buf_ptr = buf.as_mut_ptr().cast::<u8>();
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), buf_ptr, len);
            }
            Ok(Self { repr: Repr::Inline(buf, len) })
        } else {
            #[cfg(feature = "alloc")]
            {
                Ok(Self { repr: Repr::Heap(CString::from(s)) })
            }
            #[cfg(not(feature = "alloc"))]
            {
                Err(CapacityOverflowError(len))
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl<const N: usize> From<CString> for SmallCString<N> {
    #[inline]
    fn from(s: CString) -> Self {
        Self { repr: Repr::Heap(s) }
    }
}

// Turns this `SmallCString` into an empty string to prevent
// memory-unsafe code from working by accident. Inline
// to prevent LLVM from optimizing it away in debug builds.
impl<const N: usize> Drop for SmallCString<N> {
    #[inline]
    fn drop(&mut self) {
        match &mut self.repr {
            Repr::Inline(b, ..) => unsafe {
                b.as_mut_ptr().cast::<u8>().write(0_u8);
            },
            #[cfg(feature = "alloc")]
            Repr::Heap(_) => {
                // CString already do the same thing
            }
        }
    }
}

impl<const N: usize> ops::Deref for SmallCString<N> {
    type Target = CStr;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_bytes_with_nul_unchecked(self.as_bytes_with_nul()) }
    }
}

impl<const N: usize> fmt::Debug for SmallCString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

// #[cfg(feature = "alloc")]
// impl<const N: usize> From<SmallCString<N>> for Vec<u8> {
//     /// Converts a [`SmallCString`] into a <code>[Vec]<[u8]></code>.
//     ///
//     /// The conversion consumes the [`SmallCString`], and removes the terminating NUL byte.
//     #[inline]
//     fn from(s: SmallCString<N>) -> Self {
//         s.into_bytes()
//     }
// }

impl<const N: usize> Default for SmallCString<N> {
    /// Creates an empty `CString`.
    fn default() -> Self {
        static_cmp!(usize, N > 0);
        let a: &CStr = Default::default();
        // This unwrap will never fail because N > 0.
        Self::try_from(a).unwrap()
    }
}

impl<const N: usize> AsRef<CStr> for SmallCString<N> {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self
    }
}

impl<const N: usize> Borrow<CStr> for SmallCString<N> {
    #[inline]
    fn borrow(&self) -> &CStr {
        self
    }
}

impl<const N: usize> ops::Index<ops::RangeFull> for SmallCString<N> {
    type Output = CStr;
    #[inline]
    fn index(&self, _index: ops::RangeFull) -> &Self::Output {
        self
    }
}

#[cfg(feature = "alloc")]
impl<const N: usize> From<Box<CStr>> for SmallCString<N> {
    /// Converts a <code>[Box]<[CStr]></code> into a [`SmallCString`] without copying or allocating.
    ///
    /// This is equivalent to `SmallCString::from(CString::from(v))`.
    #[inline]
    fn from(s: Box<CStr>) -> Self {
        Self::from(CString::from(s))
    }
}

#[cfg(feature = "alloc")]
impl<const N: usize> From<Vec<NonZeroU8>> for SmallCString<N> {
    /// Converts a <code>[Vec]<[NonZeroU8]></code> into a [`SmallCString`] without
    /// copying nor checking for inner null bytes.
    ///
    /// This is equivalent to `SmallCString::from(CString::from(v))`.
    #[inline]
    fn from(v: Vec<NonZeroU8>) -> SmallCString<N> {
        Self::from(CString::from(v))
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
    // TODO
}
