/* SPDX-License-Identifier: Apache-2.0 OR MIT */

/* QEMU lm3s6965evb. The ELF loader initializes both FLASH and RAM. */
MEMORY
{
    FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 256K
    RAM (rw)   : ORIGIN = 0x20000000, LENGTH = 64K
}

ENTRY(_start)
_stack = ORIGIN(RAM) + LENGTH(RAM);

SECTIONS
{
    .vector_table ORIGIN(FLASH) : { KEEP(*(.vector_table)) } > FLASH
    .text : { *(.text .text.*) } > FLASH
    .rodata : { *(.rodata .rodata.*) } > FLASH
    .data : { *(.data .data.*) } > RAM
    .bss (NOLOAD) : { *(.bss .bss.* COMMON) } > RAM

    /DISCARD/ : { *(.ARM.exidx* .ARM.extab*) }
}
