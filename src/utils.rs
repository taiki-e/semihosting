// SPDX-License-Identifier: Apache-2.0 OR MIT

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

// This module provides core::ptr strict_provenance/exposed_provenance polyfill for pre-1.84 rustc.
pub(crate) mod ptr {
    cfg_sel!({
        #[cfg(not(semihosting_no_strict_provenance))]
        {
            pub(crate) use core::ptr::{with_exposed_provenance, with_exposed_provenance_mut};
        }
        #[cfg(else)]
        {
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
        }
    });
}
