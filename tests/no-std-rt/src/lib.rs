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

#[cfg(mclass)]
#[macro_export]
macro_rules! entry {
    ($entry_fn:ident) => {
        #[::cortex_m_rt::entry]
        fn main() -> ! {
            ::semihosting::process::Termination::report($entry_fn()).exit_process()
        }
    };
}
#[cfg(feature = "qemu-system")]
#[cfg(any(target_arch = "aarch64"))]
#[macro_export]
macro_rules! entry {
    ($entry_fn:ident) => {
        #[no_mangle]
        unsafe fn _start_rust() -> ! {
            ::semihosting::process::Termination::report($entry_fn()).exit_process()
        }
    };
}
#[cfg(not(all(target_arch = "aarch64", feature = "qemu-system")))]
#[cfg(not(mclass))]
#[macro_export]
macro_rules! entry {
    ($entry_fn:ident) => {
        #[no_mangle]
        unsafe fn _start() -> ! {
            unsafe { $crate::init_start() }
            ::semihosting::process::Termination::report($entry_fn()).exit_process()
        }
    };
}
#[cfg(not(all(target_arch = "aarch64", feature = "qemu-system")))]
#[cfg(not(mclass))]
#[doc(hidden)]
#[inline(always)]
pub unsafe fn init_start() {
    #[cfg(all(any(target_arch = "riscv32", target_arch = "riscv64"), feature = "qemu-system"))]
    unsafe {
        core::arch::asm!("la sp, _stack");
    }
    #[cfg(all(armv5te, feature = "qemu-system"))]
    unsafe {
        #[instruction_set(arm::a32)]
        #[inline]
        unsafe fn _init() {
            unsafe {
                // For integratorcp, musicpal, realview-eb, versatileab, and versatilepb
                core::arch::asm!("mov sp, #0x8000");
            }
        }
        _init();
    }
}
