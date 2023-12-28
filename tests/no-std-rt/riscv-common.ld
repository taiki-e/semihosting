/* SPDX-License-Identifier: Apache-2.0 OR MIT */

/* A common part of linker scripts for RISC-V test with QEMU.
 * This is intended to be included from riscv32.ld or riscv64.ld.
 */

SECTIONS
{
    . = FW_JUMP_ADDR;
    PROVIDE(__executable_start = .); /* for unwinding support */

    .text : {
        *(.text._start)
        *(.text .text.*)
    }
    PROVIDE(__etext = .); /* for unwinding support */
    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }

    /* for unwinding support */
    . = ALIGN(8);
    PROVIDE(__eh_frame = .);
    .eh_frame : { KEEP(*(.eh_frame)) *(.eh_frame.*) }

    /* https://github.com/riscv-non-isa/riscv-eabi-spec/blob/HEAD/EABI.adoc#eabi-stack-alignment */
    /* https://github.com/riscv-non-isa/riscv-elf-psabi-doc/blob/HEAD/riscv-elf.adoc#tag_riscv_stack_align-4-uleb128value */
    . = ALIGN(16);
    PROVIDE(_stack = . + 8M);
}
