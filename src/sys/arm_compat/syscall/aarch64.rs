// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::arch::asm;

use super::{OperationNumber, ParamRegR, ParamRegW, RetReg};

macro_rules! trap {
    () => {
        "hlt 0xF000" // .inst 0xD45E0000
    };
}

/// Raw semihosting call with a parameter that will be read + modified by the host.
#[inline]
pub unsafe fn syscall(number: OperationNumber, parameter: ParamRegW<'_>) -> RetReg {
    let ret;
    unsafe {
        asm!(
            trap!(),
            in("w0") number.0, // OPERATION NUMBER REGISTER
            // Use inout because operation such as SYS_ELAPSED suggest that
            // the PARAMETER REGISTER may be changed.
            inout("x1") parameter.0 => _, // PARAMETER REGISTER
            lateout("x0") ret, // RETURN REGISTER
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
            in("w0") number.0, // OPERATION NUMBER REGISTER
            // Use inout because operation such as SYS_ELAPSED suggest that
            // the PARAMETER REGISTER may be changed.
            inout("x1") parameter.0 => _, // PARAMETER REGISTER
            lateout("x0") ret, // RETURN REGISTER
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
            in("w0") number.0, // OPERATION NUMBER REGISTER
            in("x1") parameter.0, // PARAMETER REGISTER
            lateout("x0") ret, // RETURN REGISTER
            options(nostack, preserves_flags),
        );
    }
    #[cfg(debug_assertions)]
    unsafe {
        let param_new;
        asm!(
            trap!(),
            in("w0") number.0, // OPERATION NUMBER REGISTER
            inout("x1") parameter.0 => param_new, // PARAMETER REGISTER
            lateout("x0") ret, // RETURN REGISTER
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
            in("w0") number.0, // OPERATION NUMBER REGISTER
            in("x1") parameter.0, // PARAMETER REGISTER
            lateout("x0") ret, // RETURN REGISTER
            options(nostack, preserves_flags, readonly),
        );
    }
    #[cfg(debug_assertions)]
    unsafe {
        let param_new;
        asm!(
            trap!(),
            in("w0") number.0, // OPERATION NUMBER REGISTER
            inout("x1") parameter.0 => param_new, // PARAMETER REGISTER
            lateout("x0") ret, // RETURN REGISTER
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
            in("w0") number.0, // OPERATION NUMBER REGISTER
            in("x1") parameter.0, // PARAMETER REGISTER
            options(nostack, noreturn, preserves_flags, readonly),
        )
    }
}
