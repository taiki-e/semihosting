// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::mem::MaybeUninit;

// rustfmt-compatible cfg_select/cfg_if alternative
// Note: This macro is cfg_sel!({ }), not cfg_sel! { }.
// An extra brace is used in input to make contents rustfmt-able.
macro_rules! cfg_sel {
    ({#[cfg(else)] { $($output:tt)* }}) => {
        $($output)*
    };
    ({
        #[cfg($cfg:meta)]
        { $($output:tt)* }
        $($( $rest:tt )+)?
    }) => {
        #[cfg($cfg)]
        cfg_sel! {{#[cfg(else)] { $($output)* }}}
        $(
            #[cfg(not($cfg))]
            cfg_sel! {{ $($rest)+ }}
        )?
    };
}

// <[_]>::assume_init_ref requires Rust 1.93.
/// # Safety
///
/// Calling this when the content is not yet fully initialized causes undefined
/// behavior: it is up to the caller to guarantee that every `MaybeUninit<T>` in
/// the slice really is in an initialized state.
#[allow(dead_code)]
#[inline(always)]
pub(crate) const unsafe fn slice_assume_init_ref<T>(s: &[MaybeUninit<T>]) -> &[T] {
    // SAFETY: casting `slice` to a `*const [T]` is safe since the caller guarantees that
    // `slice` is initialized, and `MaybeUninit` is guaranteed to have the same layout as `T`.
    // The pointer obtained is valid since it refers to memory owned by `slice` which is a
    // reference and thus guaranteed to be valid for reads.
    unsafe { &*(s as *const [MaybeUninit<T>] as *const [T]) }
}
// <[_]>::assume_init_mut requires Rust 1.93.
/// # Safety
///
/// Calling this when the content is not yet fully initialized causes undefined
/// behavior: it is up to the caller to guarantee that every `MaybeUninit<T>` in the
/// slice really is in an initialized state. For instance, `.assume_init_mut()` cannot
/// be used to initialize a `MaybeUninit` slice.
#[inline(always)]
pub(crate) const unsafe fn slice_assume_init_mut<T>(s: &mut [MaybeUninit<T>]) -> &mut [T] {
    // SAFETY: similar to safety notes for `slice_get_ref`, but we have a
    // mutable reference which is also guaranteed to be valid for writes.
    unsafe { &mut *(s as *mut [MaybeUninit<T>] as *mut [T]) }
}

// This module provides core::ptr strict_provenance/exposed_provenance polyfill for pre-1.84 rustc.
pub(crate) mod ptr {
    cfg_sel!({
        #[cfg(not(semihosting_no_strict_provenance))]
        {
            pub(crate) use core::ptr::{
                with_exposed_provenance, with_exposed_provenance_mut, without_provenance_mut,
            };
        }
        #[cfg(else)]
        {
            use core::mem;

            #[inline(always)]
            #[must_use]
            pub(crate) const fn without_provenance_mut<T>(addr: usize) -> *mut T {
                // An int-to-pointer transmute currently has exactly the intended semantics: it creates a
                // pointer without provenance. Note that this is *not* a stable guarantee about transmute
                // semantics, it relies on sysroot crates having special status.
                // SAFETY: every valid integer is also a valid pointer (as long as you don't dereference that
                // pointer).
                unsafe { mem::transmute(addr) }
            }
            #[inline(always)]
            #[must_use]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn with_exposed_provenance<T>(addr: usize) -> *const T {
                addr as *const T
            }
            #[inline(always)]
            #[must_use]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn with_exposed_provenance_mut<T>(addr: usize) -> *mut T {
                addr as *mut T
            }

            pub(crate) trait PtrExt<T: ?Sized>: Copy {
                #[must_use]
                fn addr(self) -> usize;
            }
            impl<T: ?Sized> PtrExt<T> for *const T {
                #[inline(always)]
                #[must_use]
                fn addr(self) -> usize {
                    // A pointer-to-integer transmute currently has exactly the right semantics: it returns the
                    // address without exposing the provenance. Note that this is *not* a stable guarantee about
                    // transmute semantics, it relies on sysroot crates having special status.
                    // SAFETY: Pointer-to-integer transmutes are valid (if you are okay with losing the
                    // provenance).
                    unsafe { mem::transmute(self.cast::<()>()) }
                }
            }
            impl<T: ?Sized> PtrExt<T> for *mut T {
                #[inline(always)]
                #[must_use]
                fn addr(self) -> usize {
                    // A pointer-to-integer transmute currently has exactly the right semantics: it returns the
                    // address without exposing the provenance. Note that this is *not* a stable guarantee about
                    // transmute semantics, it relies on sysroot crates having special status.
                    // SAFETY: Pointer-to-integer transmutes are valid (if you are okay with losing the
                    // provenance).
                    unsafe { mem::transmute(self.cast::<()>()) }
                }
            }
        }
    });
}
