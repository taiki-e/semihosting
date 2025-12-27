// SPDX-License-Identifier: Apache-2.0 OR MIT

#![no_std]
#![warn(unsafe_op_in_unsafe_fn)]

#[cfg(any(
    not(any(feature = "qemu-user", feature = "qemu-system")),
    all(feature = "qemu-user", feature = "qemu-system"),
))]
compile_error!("no-std-rt: exactly one of 'qemu-user' or 'qemu-system' feature must be enabled");

#[cfg(all(target_arch = "aarch64", feature = "qemu-system"))]
#[no_mangle]
#[link_section = ".text._start_arguments"]
pub static BOOT_CORE_ID: u64 = 0;
#[cfg(all(target_arch = "aarch64", feature = "qemu-system"))]
core::arch::global_asm!(include_str!("../raspi/boot.s"));

#[cfg(cortex_m_rt)]
#[doc(hidden)]
pub use cortex_m_rt::{entry as cortex_m_rt_entry, *};
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
#[doc(hidden)]
pub use aarch32_rt::{entry as aarch32_rt_entry, *};
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
#[cfg(all(target_arch = "aarch64", feature = "qemu-system"))]
#[macro_export]
macro_rules! entry {
    ($entry_fn:ident) => {
        #[no_mangle]
        unsafe extern "C" fn _start_rust() -> ! {
            main()
        }
        fn main() -> ! {
            ::semihosting::process::Termination::report($entry_fn()).exit_process()
        }
    };
}
#[cfg(not(cortex_m_rt))]
#[cfg(not(aarch32_rt))]
#[cfg(not(all(target_arch = "aarch64", feature = "qemu-system")))]
#[macro_export]
macro_rules! entry {
    ($entry_fn:ident) => {
        #[no_mangle]
        unsafe extern "C" fn _start() -> ! {
            unsafe { $crate::init_start() }
            main()
        }
        fn main() -> ! {
            ::semihosting::process::Termination::report($entry_fn()).exit_process()
        }
    };
}
#[cfg(not(cortex_m_rt))]
#[cfg(not(aarch32_rt))]
#[cfg(not(all(target_arch = "aarch64", feature = "qemu-system")))]
#[doc(hidden)]
#[inline(always)]
pub unsafe fn init_start() {
    #[cfg(all(any(target_arch = "riscv32", target_arch = "riscv64"), feature = "qemu-system"))]
    unsafe {
        core::arch::asm!("la sp, _stack");
    }
}
