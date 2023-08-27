/* SPDX-License-Identifier: Apache-2.0 OR MIT */

/* Linker script for ARMv4T test with QEMU */

ENTRY(_start)

SECTIONS
{

    . = 0x0000;

    .text : {
        *(.text._start)
        *(.text .text.*)
    }
}
