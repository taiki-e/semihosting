// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::arch::asm;

use super::{OperationNumber, ParamRegR, ParamRegW, RetReg};

// Semihosting Trap Instruction
cfg_sel!({
    // M-profile architecture
    #[cfg(any(target_feature = "mclass", semihosting_target_feature = "mclass"))]
    {
        // T32
        macro_rules! trap {
            () => {
                "bkpt 0xAB" // .inst 0xBEAB
            };
        }
    }
    // HLT
    // https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#the-semihosting-interface
    // > This requirement includes supporting the HLT encodings on ARMv7 and earlier processors,
    // > even though HLT is only defined as an instruction in ARMv8. This may require the semihosting
    // > implementation to trap the UNDEF exception.
    // >
    // > The HLT encodings are new in version 2.0 of the semihosting specification. Where possible,
    // > have semihosting callers continue to use the previously existing trap instructions to ensure
    // > compatibility with legacy semihosting implementations. These trap instructions are HLT for A64,
    // > SVC on A+R profile A32 or T32, and BKPT on M profile. However, it is necessary to change from
    // > SVC to HLT instructions to support AArch32 semihosting properly in a mixed AArch32/AArch64 system.
    // >
    // > ARM encourages semihosting callers to implement support for trapping using HLT on A32 and T32
    // > as a configurable option. ARM strongly discourages semihosting callers from mixing the HLT and
    // > SVC mechanisms within the same executable.
    #[cfg(feature = "trap-hlt")]
    {
        // T32
        #[cfg(any(target_feature = "thumb-mode", semihosting_target_feature = "thumb-mode"))]
        macro_rules! trap {
            () => {
                ".inst 0xBABC" // hlt 0x3C
            };
        }
        // A32
        #[cfg(not(any(target_feature = "thumb-mode", semihosting_target_feature = "thumb-mode")))]
        macro_rules! trap {
            () => {
                ".inst 0xE10F0070" // hlt 0xF000
            };
        }
    }
    // SVC
    #[cfg(else)]
    {
        // T32
        #[cfg(any(target_feature = "thumb-mode", semihosting_target_feature = "thumb-mode"))]
        macro_rules! trap {
            () => {
                "svc 0xAB" // .inst 0xDFAB
            };
        }
        // A32
        #[cfg(not(any(target_feature = "thumb-mode", semihosting_target_feature = "thumb-mode")))]
        macro_rules! trap {
            () => {
                "svc 0x123456" // .inst 0xEF123456
            };
        }
    }
});

/// Raw semihosting call with a parameter that will be read + modified by the host.
#[inline]
pub unsafe fn syscall(number: OperationNumber, parameter: ParamRegW<'_>) -> RetReg {
    let ret;
    unsafe {
        asm!(
            trap!(),
            inout("r0") number.0 => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            // Use inout because operation such as SYS_ELAPSED suggest that
            // PARAMETER REGISTER may be changed.
            inout("r1") parameter.0 => _, // PARAMETER REGISTER
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
            inout("r0") number.0 => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            // Use inout because operation such as SYS_ELAPSED suggest that
            // PARAMETER REGISTER may be changed.
            inout("r1") parameter.0 => _, // PARAMETER REGISTER
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
            inout("r0") number.0 => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            in("r1") parameter.0, // PARAMETER REGISTER
            options(nostack, preserves_flags),
        );
    }
    #[cfg(debug_assertions)]
    unsafe {
        let param_new;
        asm!(
            trap!(),
            inout("r0") number.0 => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            inout("r1") parameter.0 => param_new, // PARAMETER REGISTER
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
            inout("r0") number.0 => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            in("r1") parameter.0, // PARAMETER REGISTER
            options(nostack, preserves_flags, readonly),
        );
    }
    #[cfg(debug_assertions)]
    unsafe {
        let param_new;
        asm!(
            trap!(),
            inout("r0") number.0 => ret, // OPERATION NUMBER REGISTER => RETURN REGISTER
            inout("r1") parameter.0 => param_new, // PARAMETER REGISTER
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
            in("r0") number.0, // OPERATION NUMBER REGISTER
            in("r1") parameter.0, // PARAMETER REGISTER
            options(nostack, noreturn, preserves_flags, readonly),
        )
    }
}
