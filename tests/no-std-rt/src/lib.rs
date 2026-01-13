// SPDX-License-Identifier: Apache-2.0 OR MIT

#![no_std]
#![warn(unsafe_op_in_unsafe_fn)]
#![cfg_attr(
    any(
        target_arch = "mips",
        target_arch = "mips32r6",
        target_arch = "mips64",
        target_arch = "mips64r6",
    ),
    feature(asm_experimental_arch)
)]

#[cfg(any(
    not(any(feature = "qemu-user", feature = "qemu-system")),
    all(feature = "qemu-user", feature = "qemu-system"),
))]
compile_error!("no-std-rt: exactly one of 'qemu-user' or 'qemu-system' feature must be enabled");

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

// Note: cannot use cfg_sel together with #[macro_export] macros.
#[cfg(cortex_m_rt)]
#[macro_export]
macro_rules! entry {
    ($entry_fn:ident) => {
        extern crate semihosting_no_std_test_rt as cortex_m_rt;
        #[::semihosting_no_std_test_rt::cortex_m_rt_entry]
        fn main() -> ! {
            ::semihosting::process::Termination::report($entry_fn()).exit_process()
        }
    };
}
#[cfg(aarch32_rt)]
#[macro_export]
macro_rules! entry {
    ($entry_fn:ident) => {
        extern crate semihosting_no_std_test_rt as aarch32_rt;
        #[::semihosting_no_std_test_rt::aarch32_rt_entry]
        fn main() -> ! {
            ::semihosting::process::Termination::report($entry_fn()).exit_process()
        }
    };
}
#[cfg(not(cortex_m_rt))]
#[cfg(not(aarch32_rt))]
#[macro_export]
macro_rules! entry {
    ($entry_fn:ident) => {
        #[no_mangle]
        pub extern "C" fn _start_rust() -> ! {
            ::semihosting::process::Termination::report($entry_fn()).exit_process()
        }
    };
}
#[cfg(not(cortex_m_rt))]
#[cfg(not(aarch32_rt))]
extern "C" {
    fn _start_rust() -> !;
}

cfg_sel!({
    #[cfg(cortex_m_rt)]
    {
        #[doc(hidden)]
        pub use cortex_m_rt::{entry as cortex_m_rt_entry, *};
    }
    #[cfg(aarch32_rt)]
    {
        #[doc(hidden)]
        pub use aarch32_rt::{entry as aarch32_rt_entry, *};
    }
    #[cfg(feature = "qemu-user")]
    {
        #[doc(hidden)]
        #[no_mangle]
        pub unsafe fn _start() -> ! {
            unsafe { _start_rust() }
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        #[no_mangle]
        #[link_section = ".text._start_arguments"]
        pub static BOOT_CORE_ID: u64 = 0;
        core::arch::global_asm!(include_str!("../raspi/boot.s"));
    }
    #[cfg(else)]
    {
        #[doc(hidden)]
        #[no_mangle]
        pub unsafe fn _start() -> ! {
            cfg_sel!({
                #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
                {
                    unsafe {
                        core::arch::asm!(
                            "
                            la sp, _stack
                            j _start_rust
                            ",
                            options(noreturn),
                        )
                    }
                }
                #[cfg(any(target_arch = "loongarch32", target_arch = "loongarch64"))]
                {
                    unsafe {
                        core::arch::asm!(
                            "
                            // Refs: https://github.com/enkerewpo/baremetal-loongarch64-unwinding-test
                            li.w $r12, 0x01 // FPE=1, SXE=0, ASXE=0, BTE=0
                            csrwr $r12, 0x02
                            la.abs $sp, _stack
                            b _start_rust
                            ",
                            options(noreturn),
                        )
                    }
                }
                #[cfg(any(
                    target_arch = "mips",
                    target_arch = "mips32r6",
                    target_arch = "mips64",
                    target_arch = "mips64r6",
                ))]
                {
                    unsafe {
                        core::arch::asm!(
                            "
                            .set push
                            .set noat
                            la $sp, _stack
                            b _start_rust
                            .set pop
                            ",
                            options(noreturn),
                        )
                    }
                }
                #[cfg(else)]
                {
                    unsafe { _start_rust() }
                }
            });
        }
    }
});
