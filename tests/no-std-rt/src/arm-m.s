/* SPDX-License-Identifier: Apache-2.0 OR MIT */

.syntax unified
.thumb

.section .vector_table, "a", %progbits
    .word _stack
    .word _start
    /* The tests do not enable device interrupts. */
    .rept 14
    .word .Lexception
    .endr

.section .text._start, "ax", %progbits
.global _start
.thumb_func
_start:
.ifdef .Larm_hard_float
    /* Enable CP10 and CP11 in CPACR before running Rust. */
    ldr r0, =0xe000ed88
    ldr r1, [r0]
    orr r1, r1, #0x00f00000
    str r1, [r0]
    dsb
    isb
.endif
    bl _start_rust

.thumb_func
.Lexception:
    b .Lexception
