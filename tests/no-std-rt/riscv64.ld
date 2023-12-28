/* SPDX-License-Identifier: Apache-2.0 OR MIT */

/* Linker script for RISC-V RV64 test with QEMU */

ENTRY(_start)

/* https://github.com/riscv-software-src/opensbi/blob/v1.2/platform/generic/objects.mk#L25-L34 */
FW_JUMP_ADDR = 0x80000000 + 0x200000;

INCLUDE riscv-common.ld
