// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::arch::asm;

use super::{OperationNumber, ParamRegR, ParamRegW, RetReg};

// TODO: cast in `number.0 as usize` is probably needless per
// https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#the-semihosting-interface
// > The operation number is passed in W0 for the 64-bit ABI, which is the
// > bottom 32 bits of the 64-bit register X0. Semihosting implementations must
// > not assume that the top 32 bits of X0 are 0.

macro_rules! trap {
    () => {
        "dbcl 0xab"
    };
}

/// Raw semihosting call with a parameter that will be read + modified by the host.
#[inline]
pub unsafe fn syscall(number: OperationNumber, parameter: ParamRegW<'_>) -> RetReg {
    let ret;
    unsafe {
        asm!(
            trap!(),
            inout("$a0") number.0 as usize => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            // Use inout because operation such as SYS_ELAPSED suggest that
            // PARAMETER REGISTER may be changed.
            inout("$a1") parameter.0 => _, // PARAMETER REGISTER
            options(nostack, preserves_flags),
        );
    }
    RetReg(ret)
}

/// Raw semihosting call with a parameter that will be read (but not modified) by the host.
#[inline]
pub unsafe fn syscall_readonly(number: OperationNumber, parameter: ParamRegR<'_>) -> RetReg {
    let ret;
    unsafe {
        asm!(
            trap!(),
            inout("$a0") number.0 as usize => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            // Use inout because operation such as SYS_ELAPSED suggest that
            // PARAMETER REGISTER may be changed.
            inout("$a1") parameter.0 => _, // PARAMETER REGISTER
            options(nostack, preserves_flags, readonly),
        );
    }
    RetReg(ret)
}

#[inline]
pub(crate) unsafe fn syscall_param_unchanged(
    number: OperationNumber,
    parameter: ParamRegW<'_>,
) -> RetReg {
    let ret;
    #[cfg(not(debug_assertions))]
    unsafe {
        asm!(
            trap!(),
            inout("$a0") number.0 as usize => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            in("$a1") parameter.0, // PARAMETER REGISTER
            options(nostack, preserves_flags),
        );
    }
    #[cfg(debug_assertions)]
    unsafe {
        let param_new;
        asm!(
            trap!(),
            inout("$a0") number.0 as usize => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            inout("$a1") parameter.0 => param_new, // PARAMETER REGISTER
            options(nostack, preserves_flags),
        );
        assert_eq!(parameter.0, param_new);
    }
    RetReg(ret)
}

#[inline]
pub(crate) unsafe fn syscall_param_unchanged_readonly(
    number: OperationNumber,
    parameter: ParamRegR<'_>,
) -> RetReg {
    let ret;
    #[cfg(not(debug_assertions))]
    unsafe {
        asm!(
            trap!(),
            inout("$a0") number.0 as usize => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            in("$a1") parameter.0, // PARAMETER REGISTER
            options(nostack, preserves_flags, readonly),
        );
    }
    #[cfg(debug_assertions)]
    unsafe {
        let param_new;
        asm!(
            trap!(),
            inout("$a0") number.0 as usize => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            inout("$a1") parameter.0 => param_new, // PARAMETER REGISTER
            options(nostack, preserves_flags, readonly),
        );
        assert_eq!(parameter.0, param_new);
    }
    RetReg(ret)
}

#[inline]
pub(crate) unsafe fn syscall_noreturn_readonly(
    number: OperationNumber,
    parameter: ParamRegR<'_>,
) -> ! {
    unsafe {
        asm!(
            trap!(),
            // An infinite loop to prevent the noreturn contract from being violated when a
            // semihosting call doesn't work for some reason.
            "2:",
                "b 2b",
            in("$a0") number.0 as usize, // OPERATION NUMBER REGISTER
            in("$a1") parameter.0, // PARAMETER REGISTER
            options(nostack, noreturn, preserves_flags, readonly),
        )
    }
}
