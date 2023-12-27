// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::arch::asm;

use super::{OperationNumber, ParamRegR, ParamRegW, RetReg};

// Semihosting Trap Instruction
#[cfg(any(target_feature = "mclass", semihosting_target_feature = "mclass"))]
macro_rules! trap {
    () => {
        "bkpt 0xAB"
    };
}
// #[cfg(not(semihosting_arm_trap_hlt))]
#[cfg(not(any(target_feature = "mclass", semihosting_target_feature = "mclass")))]
#[cfg(any(target_feature = "thumb-mode", semihosting_target_feature = "thumb-mode"))]
macro_rules! trap {
    () => {
        "svc 0xAB"
    };
}
// #[cfg(not(semihosting_arm_trap_hlt))]
#[cfg(not(any(target_feature = "mclass", semihosting_target_feature = "mclass")))]
#[cfg(not(any(target_feature = "thumb-mode", semihosting_target_feature = "thumb-mode")))]
macro_rules! trap {
    () => {
        "svc 0x123456"
    };
}
// #[cfg(semihosting_arm_trap_hlt)]
// #[cfg(not(any(target_feature = "mclass", semihosting_target_feature = "mclass")))]
// #[cfg(any(target_feature = "thumb-mode", semihosting_target_feature = "thumb-mode"))]
// macro_rules! trap {
//     () => {
//         "hlt 0x3C"
//     };
// }
// #[cfg(semihosting_arm_trap_hlt)]
// #[cfg(not(any(target_feature = "mclass", semihosting_target_feature = "mclass")))]
// #[cfg(not(any(target_feature = "thumb-mode", semihosting_target_feature = "thumb-mode")))]
// macro_rules! trap {
//     () => {
//         "hlt 0xF000"
//     };
// }

/// Raw semihosting call with a parameter that will be read + modified by the host
#[inline]
pub unsafe fn syscall(number: OperationNumber, parameter: ParamRegW<'_>) -> RetReg {
    unsafe {
        let r;
        asm!(
            trap!(),
            inout("r0") number.0 as usize => r, // OPERATION NUMBER REGISTER => RETURN REGISTER
            // Use inout because operation such as SYS_ELAPSED suggest that
            // PARAMETER REGISTER may be changed.
            inout("r1") parameter.0 => _, // PARAMETER REGISTER
            options(nostack, preserves_flags),
        );
        RetReg(r)
    }
}

/// Raw semihosting call with a parameter that will be read (but not modified) by the host
#[inline]
pub unsafe fn syscall_readonly(number: OperationNumber, parameter: ParamRegR<'_>) -> RetReg {
    unsafe {
        let r;
        asm!(
            trap!(),
            inout("r0") number.0 as usize => r, // OPERATION NUMBER REGISTER => RETURN REGISTER
            // Use inout because operation such as SYS_ELAPSED suggest that
            // PARAMETER REGISTER may be changed.
            inout("r1") parameter.0 => _, // PARAMETER REGISTER
            options(nostack, preserves_flags, readonly),
        );
        RetReg(r)
    }
}
