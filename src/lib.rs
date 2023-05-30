// SPDX-License-Identifier: Apache-2.0 OR MIT

/*!
<!-- tidy:crate-doc:start -->
Semihosting for AArch64, ARM, RISC-V (RV32 & RV64), MIPS, and MIPS64.

This library provides access to semihosting, a mechanism for programs running on the real or virtual (e.g., QEMU) target to communicate with I/O facilities on the host system. See the [ARM documentation](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst) for more information on semihosting.

APIs are categorized into the following four types:

- The top-level API (`semihosting::{io,fs,..}`) provides a subset of the standard library's similar APIs.
  - `io`: Provide no-std io traits and `std{in,out,err}`. (`std{in,out,err}` requires `stdio` feature)
  - `fs`: Provide methods to manipulate the contents of the host filesystem. (requires `fs` feature)
  - `process`: Provide `abort` and `exit`.
  - `dbg!`/`print{,ln}!`/`eprint{,ln}!`: macros to output to stdout/stderr. (requires `stdio` feature)

  Note that some APIs are not strictly a subset of the standard library.
  - API that uses types not available in `core` such as `Path` (technically, the same thing could be implemented, but it makes sense to use `CStr` directly, because when converting a long `Path`/`OsStr` to `CStr`, it needs to either [do an allocation](https://github.com/rust-lang/rust/blob/1.69.0/library/std/src/sys/common/small_c_string.rs#L30-L32) or return an error)
  - API that panics on failure in `std` (in no-std it makes sense to return `Result` since `panic=abort` is default)

- Helpers that are useful when using this library.
  - `c!`: `CStr` literal macro.

- `semihosting::sys` module, which provides low-level access to platform-specific semihosting interfaces.

- `semihosting::experimental` module, which provides experimental APIs. See [optional features](#optional-features) for more.

Additionally, this library provides a panic handler for semihosting and `-C panic=unwind` support, via optional features.

## Platform Support

The following target architectures are supported:

| target_arch | Specification | `semihosting::sys` module |
| ----------- | ------------- | ------------------------- |
| arm/aarch64 | [Semihosting for AArch32 and AArch64](https://github.com/ARM-software/abi-aa/blob/HEAD/semihosting/semihosting.rst) | `sys::arm_compat` |
| riscv32/riscv64 | [RISC-V Semihosting](https://github.com/riscv-software-src/riscv-semihosting/blob/HEAD/riscv-semihosting-spec.adoc) | `sys::arm_compat` |
| mips/mips64 | Unified Hosting Interface (MD01069) | `sys::mips` |

The host must be running an emulator or a debugger attached to the target.

The following targets have been tested on CI. (qemu-system has been tested on Linux, macOS, and Windows hosts, and qemu-user on Linux host.)

| target                               | exit | all-apis \[1] (qemu-system) | all-apis \[1] (qemu-user) | panic-unwind (qemu-system \[2]) |
| ------------------------------------ | ---- | --------------------------- | ------------------------- | ------------------------------- |
| `aarch64-unknown-none{,-softfloat}`  | ✓    | ✓                           | ✓                         | ✓                               |
| `{arm,thumb}v4t-none-eabi`           | ✓    |                             | ✓                         |                                 |
| `{arm,thumb}v5te-none-eabi`          | ✓    | ✓                           | ✓                         |                                 |
| `armv7a-none-eabi{,hf}`              | ✓    | ✓                           | ✓                         |                                 |
| `armv7r-none-eabi{,hf}`              | ✓    | ✓                           | ✓                         |                                 |
| `armebv7r-none-eabi{,hf}`            | ✓    |                             | ✓                         |                                 |
| `thumbv6m-none-eabi`                 | ✓    | ✓                           | N/A                       |                                 |
| `thumbv7m-none-eabi`                 | ✓    | ✓                           | N/A                       |                                 |
| `thumbv7em-none-eabi{,hf}`           | ✓    | ✓                           | N/A                       |                                 |
| `thumbv8m.base-none-eabi`            | ✓    | ✓ \[3]                      | N/A                       |                                 |
| `thumbv8m.main-none-eabi{,hf}`       | ✓    | ✓ \[3]                      | N/A                       |                                 |
| `riscv32*-unknown-none-elf`          | ✓    | ✓                           | ✓                         | ✓                               |
| `riscv64*-unknown-none-elf`          | ✓    | ✓                           | ✓                         | ✓                               |
| `mips{,el}-unknown-none` \[5]        | ✓    | ✓ \[6]                      | N/A                       |                                 |
| `mips64{,el}-unknown-none` \[5]      | ✓    | ✓ \[6]                      | N/A                       |                                 |

\[1] `stdio`, `fs`, `time`, and `args`.<br>
\[2] I'm not sure how to test panic-unwind on qemu-user.<br>
\[4] Worked on QEMU 6.2 and QEMU 8.0, failed on QEMU 7.2.<br>
\[5] Requires nightly due to `#![feature(asm_experimental_arch)]`.<br>
\[6] It seems unsupported on QEMU 8.0.<br>

## Optional features

All features are disabled by default.

In general use cases, you probably only need the `stdio` feature that enables print-related macros and/or the `panic-handler` feature that exits with a non-zero error code on panic.

```toml
[dependencies]
semihosting = { version = "0.1", features = ["stdio", "panic-handler"] }
```

- **`alloc`**<br>
  Use `alloc`.

- **`stdio`**<br>
  Enable `semihosting::io::{stdin,stdout,stderr}` and `semihosting::{print*,eprint*,dbg}`.

- **`fs`**<br>
  Enable `semihosting::fs`.

- **`panic-handler`**<br>
  Provide panic handler based on `semihosting::process::exit`.

  If the `stdio` feature is also enabled, this attempt to output panic message and
  location to stderr.

- **`portable-atomic`**<br>
  Use [portable-atomic]'s atomic types.

  portable-atomic provides atomic CAS on targets where the standard library does not provide atomic CAS.
  To use the `panic-unwind` feature on such targets (e.g., RISC-V without A-extension), you need to enable this feature.

  See [the documentation](https://github.com/taiki-e/portable-atomic#optional-cfg) for details.

- **`args`**<br>
  Enable `semihosting::experimental::env::args`.

  Note:
  - This feature is experimental (tracking issue: [#1](https://github.com/taiki-e/semihosting/issues/1))
    and outside of the normal semver guarantees and minor or patch versions of semihosting may make
    breaking changes to them at any time.

- **`time`**<br>
  Enable `semihosting::experimental::time`.

  Note:
  - This feature is experimental (tracking issue: [#2](https://github.com/taiki-e/semihosting/issues/2))
    and outside of the normal semver guarantees and minor or patch versions of semihosting may make
    breaking changes to them at any time.

- **`panic-unwind`**<br>
  Provide `-C panic=unwind` support for panic handler and enable
  `semihosting::experimental::panic::catch_unwind`.

  This currently uses [unwinding] crate to support unwinding.
  See its documentation for supported platforms and requirements.

  Note:
  - This feature is experimental (tracking issue: [#3](https://github.com/taiki-e/semihosting/issues/3))
    and outside of the normal semver guarantees and minor or patch versions of semihosting may make
    breaking changes to them at any time.
  - This requires nightly compiler.
  - This implicitly enables the `alloc` and `panic-handler` features.
  - This uses atomic CAS. You need to use `portable-atomic` feature together if your target doesn't support atomic CAS (e.g., RISC-V without A-extension).
  - When enabling this feature, you may need to rebuild the standard library with
    `-C panic=unwind` for `catch_unwind` to work properly. The recommended way to
    rebuild the standard library is passing `-Z build-std="core,alloc"` option to cargo.

- **`backtrace`**<br>
  Provide backtrace support for panic handler.

  This currently uses [unwinding] crate to support backtrace.
  See its documentation for supported platforms and requirements.

  Note:
  - This feature is experimental (tracking issue: [#3](https://github.com/taiki-e/semihosting/issues/3))
    and outside of the normal semver guarantees and minor or patch versions of semihosting may make
    breaking changes to them at any time.
  - This requires nightly compiler.
  - This implicitly enables the `stdio` feature.
  - When enabling this, it is recommended to also enable the `panic-unwind` feature. Otherwise, a decent backtrace will not be displayed at this time. (Using [`-C force-unwind-tables`](https://doc.rust-lang.org/rustc/codegen-options/index.html#force-unwind-tables) may work, but has not been tested yet.)
  - Currently, the backtrace generated is not human-readable.

    ```text
    panicked at 'a', src/main.rs:86:13
    stack backtrace:
      0x84dc0
      0x8ed80
      0x8332c
      0x83654
      0x80644
      0x803cc
      0x809dc
      0x800bc
    ```

    You can use `addr2line` to resolve the addresses and [rustfilt] to demangle Rust symbols.
    For example, run the following command (please replace `<path/to/binary>` with your binary path), then paste the addresses:

    ```sh
    llvm-addr2line -fipe <path/to/binary> | rustfilt
    ```

[portable-atomic]: https://github.com/taiki-e/portable-atomic
[rustfilt]: https://github.com/luser/rustfilt
[unwinding]: https://github.com/nbdd0121/unwinding

<!-- tidy:crate-doc:end -->
*/

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
    // clippy::undocumented_unsafe_blocks, // TODO
    // misc
    clippy::inline_asm_x86_att_syntax,
    // clippy::missing_inline_in_public_items,
)]
#![allow(
    clippy::borrow_as_ptr, // https://github.com/rust-lang/rust-clippy/issues/8286
    clippy::cast_lossless,
    clippy::doc_markdown,
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
