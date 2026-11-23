/* SPDX-License-Identifier: Apache-2.0 OR MIT */

/* QEMU VersatilePB/MPS3-AN536. The ELF loader initializes .data and zeroes .bss at their run addresses. */
ENTRY(_start)

SECTIONS
{
    .text CODE_ADDR : {
        KEEP(*(.text._start))
        *(.text .text.*)
    }
    .rodata : { *(.rodata .rodata.*) }
    ASSERT(. <= CODE_END, "code exceeds its memory region")

    /* VersatilePB uses the same RAM for code and data, MPS3 uses QSPI and DDR. */
    . = MAX(., RAM_ADDR);
    .data : { *(.data .data.*) }
    .bss (NOLOAD) : { *(.bss .bss.* COMMON) }
    _stack = RAM_END;
    ASSERT(. <= _stack - 1M, "data overlaps the stack")

    /DISCARD/ : {
        *(.ARM.exidx* .gnu.linkonce.armexidx.*)
        *(.ARM.extab* .gnu.linkonce.armextab.*)
    }
}
