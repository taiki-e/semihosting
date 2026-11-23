// SPDX-License-Identifier: Apache-2.0 OR MIT

.section .text._start,"ax",%progbits
.arm
.p2align 5
.global _start
.type _start, %function
_start:
    b .Lreset
    .rept 7
        b .Lexception
    .endr

.Lexception:
    b .Lexception

.Lreset:
.ifdef .Larm_v8r
    // Cortex-R52 boots in Hyp mode. Enter System mode in Arm state with
    // IRQ and FIQ masked, preserving QEMU's data endianness for BE8 ELFs.
    mrs r0, cpsr
    and r1, r0, #0x1f
    cmp r1, #0x1a
    bne .Lel1
    ldr r1, =_start
    mcr p15, 4, r1, c12, c0, 0 // HVBAR
    and r0, r0, #0x200 // CPSR.E
    orr r0, r0, #0xdf
    msr spsr_hyp, r0
    adr r0, .Lel1
    msr elr_hyp, r0
    isb
    eret
.Lel1:
    ldr r0, =_start
    mcr p15, 0, r0, c12, c0, 0 // VBAR
.endif

    msr cpsr_c, #0xdf // System mode, IRQ and FIQ masked
    ldr sp, =_stack
.ifdef .Larm_hard_float
    // Work around https://github.com/rust-lang/rust/issues/127269.
    .fpu vfp2
    mrc p15, 0, r0, c1, c0, 2
    orr r0, r0, #0xf00000 // Allow access to CP10 and CP11.
    mcr p15, 0, r0, c1, c0, 2
.ifdef .Larm_v6
    mcr p15, 0, r0, c7, c5, 4 // ISB on Armv6.
.else
    isb
.endif
    mov r0, #0x40000000
    vmsr fpexc, r0
.endif

    // BX enters the state indicated by the Rust entry symbol's low bit.
    ldr r0, =_start_rust
    bx r0
.size _start, . - _start
.ltorg
