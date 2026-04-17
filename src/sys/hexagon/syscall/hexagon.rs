// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::arch::asm;

use super::{OperationNumber, ParamRegR, ParamRegW, RetReg};

macro_rules! trap {
    () => {
        "trap0(#0)"
    };
}

/// Raw semihosting call with a parameter that will be read + modified by the host.
#[inline]
pub unsafe fn syscall(number: OperationNumber, parameter: ParamRegW<'_>) -> RetReg {
    let ret;
    unsafe {
        asm!(
            trap!(),
            inout("r0") number.0 => ret,
            inout("r1") parameter.0 => _, // parameter => err
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
            inout("r0") number.0 => ret,
            inout("r1") parameter.0 => _, // parameter => err
            options(nostack, preserves_flags, readonly),
        );
    }
    RetReg(ret)
}

// syscall2?
/// Raw semihosting call with a parameter that will be read + modified by the host.
#[inline]
pub unsafe fn direct_syscall(
    number: OperationNumber,
    arg1: ParamRegW<'_>,
    arg2: ParamRegW<'_>,
) -> RetReg {
    let ret;
    unsafe {
        asm!(
            trap!(),
            inout("r0") number.0 => ret,
            inout("r1") arg1.0 => _, // arg1 => err
            inout("r2") arg2.0 => _,
            options(nostack, preserves_flags),
        );
    }
    RetReg(ret)
}

// syscall2_readonly?
/// Raw semihosting call with a parameter that will be read (but not modified) by the host.
#[inline]
pub unsafe fn direct_syscall_readonly(
    number: OperationNumber,
    arg1: ParamRegR<'_>,
    arg2: ParamRegR<'_>,
) -> RetReg {
    let ret;
    unsafe {
        asm!(
            trap!(),
            inout("r0") number.0 => ret,
            inout("r1") arg1.0 => _, // arg1 => err
            inout("r2") arg2.0 => _,
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
                "jump 2b",
            in("r0") number.0,
            in("r1") parameter.0,
            options(nostack, noreturn, preserves_flags, readonly),
        )
    }
}
