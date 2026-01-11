// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::arch::asm;

use super::{OperationNumber, ParamRegR, ParamRegW, RetReg};

macro_rules! trap {
    () => {
        "break 1, 14"
    };
}

/// Raw semihosting call with a parameter that will be read + modified by the host.
#[inline]
pub unsafe fn syscall(number: OperationNumber, parameter: ParamRegW<'_>) -> RetReg {
    let ret;
    unsafe {
        asm!(
            trap!(),
            inout("a2") number.0 => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            // Use inout because operation such as SYS_ELAPSED suggest that
            // PARAMETER REGISTER may be changed.
            inout("a3") parameter.0 => _, // PARAMETER REGISTER
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
            inout("a2") number.0 => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            // Use inout because operation such as SYS_ELAPSED suggest that
            // PARAMETER REGISTER may be changed.
            inout("a3") parameter.0 => _, // PARAMETER REGISTER
            options(nostack, preserves_flags, readonly),
        );
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
                "j 2b",
            in("a2") number.0, // OPERATION NUMBER REGISTER
            in("a3") parameter.0, // PARAMETER REGISTER
            options(nostack, noreturn, preserves_flags, readonly),
        )
    }
}
