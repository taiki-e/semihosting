// SPDX-License-Identifier: Apache-2.0 OR MIT

#![doc = include_str!("../README.md")]
#![no_std]
#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms, single_use_lifetimes),
        allow(dead_code, unused_variables)
    )
))]
#![warn(
    improper_ctypes,
    missing_debug_implementations,
    // missing_docs,
    rust_2018_idioms,
    single_use_lifetimes,
    unreachable_pub,
    unsafe_op_in_unsafe_fn
)]
#![warn(
    clippy::pedantic,
    // lints for public library
    clippy::alloc_instead_of_core,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    // lints that help writing unsafe code
    clippy::as_ptr_cast_mut,
    clippy::default_union_representation,
    clippy::trailing_empty_array,
    clippy::transmute_undefined_repr,
    // clippy::undocumented_unsafe_blocks,
    // misc
    clippy::inline_asm_x86_att_syntax,
    // clippy::missing_inline_in_public_items,
)]
#![allow(
    clippy::borrow_as_ptr, // https://github.com/rust-lang/rust-clippy/issues/8286
    clippy::cast_lossless,
    clippy::doc_markdown,
    clippy::let_underscore_untyped, // https://github.com/rust-lang/rust-clippy/issues/10410
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_inception,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::naive_bytecount,
    clippy::similar_names,
    clippy::single_match,
    clippy::struct_excessive_bools,
    clippy::type_complexity,
    clippy::unreadable_literal,
    clippy::used_underscore_binding,
)]
#![cfg_attr(
    not(any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "arm",
        target_arch = "aarch64",
        target_arch = "riscv32",
        target_arch = "riscv64",
    )),
    feature(asm_experimental_arch)
)]
#![cfg_attr(semihosting_unstable_rustc_attrs, feature(rustc_attrs))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::empty_loop)] // this crate is #![no_std]
#![allow(clippy::len_without_is_empty, clippy::new_without_default)]

#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv32",
    target_arch = "riscv64",
    target_arch = "mips",
    target_arch = "mips64",
)))]
compile_error!("unsupported target");

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(doc)]
extern crate self as semihosting;
#[cfg(test)]
extern crate std;

#[cfg(feature = "panic-unwind")]
#[cfg(not(feature = "portable-atomic"))]
use core::sync::atomic;
#[cfg(feature = "panic-unwind")]
#[cfg(feature = "portable-atomic")]
use portable_atomic as atomic;

#[macro_use]
mod macros;

#[macro_use]
mod c_str;

#[macro_use]
pub mod fd;

#[macro_use]
pub mod io;

#[cfg(any(feature = "args", feature = "panic-unwind", feature = "time"))]
// Skip doc(cfg) due to rustdoc doesn't handle nested doc(cfg) well.
// #[cfg_attr(docsrs, doc(cfg(any(feature = "args", feature = "panic-unwind", feature = "time"))))]
pub mod experimental;
#[cfg(feature = "fs")]
#[cfg_attr(docsrs, doc(cfg(feature = "fs")))]
pub mod fs;
#[cfg(feature = "panic-handler")]
mod panicking;
pub mod process;
pub mod sys;

#[cfg(feature = "stdio")]
mod sealed {
    pub trait Sealed {}
}

// Not public API.
#[doc(hidden)]
pub mod __private {
    pub use core::{
        ffi::CStr,
        file, line,
        result::Result::{Err, Ok},
        stringify, write, writeln,
    };

    pub use crate::c_str::const_c_str_check;
}
