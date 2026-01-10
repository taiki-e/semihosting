// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::arch::asm;

use super::{OperationCode, ParamRegR, ParamRegW};

// Semihosting Trap Instruction
// ColdFire
macro_rules! trap {
    () => {
        // "halt"
        ".2byte 0x4ac8"
    };
}
// Others
// macro_rules! trap {
//     () => {
//         "bkpt #0"
//     };
// }

/// Raw semihosting call with a parameter that will be read + modified by the host
#[inline]
pub unsafe fn syscall(op: OperationCode, parameter: ParamRegW<'_>) {
    unsafe {
        asm!(
            ".balign 4",
            "nop",
            trap!(),
            ".4byte 0x4e7bf000",
            in("d0") op.0 as u32,
            in("d1") parameter.0,
            options(nostack, preserves_flags),
        );
    }
}

/// Raw semihosting call with a parameter that will be read (but not modified) by the host
#[inline]
pub unsafe fn syscall_readonly(op: OperationCode, parameter: ParamRegR<'_>) {
    unsafe {
        asm!(
            ".balign 4",
            "nop",
            trap!(),
            ".4byte 0x4e7bf000",
            in("d0") op.0 as u32,
            in("d1") parameter.0,
            options(nostack, preserves_flags, readonly),
        );
    }
}
