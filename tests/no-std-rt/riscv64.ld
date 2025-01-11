/* SPDX-License-Identifier: Apache-2.0 OR MIT */

/* Linker script for RISC-V RV64 test with QEMU */

ENTRY(_start)

/* https://github.com/riscv-software-src/opensbi/blob/v1.6/platform/template/objects.mk#L75 */
FW_JUMP_ADDR = 0x80200000;

INCLUDE riscv-common.ld
