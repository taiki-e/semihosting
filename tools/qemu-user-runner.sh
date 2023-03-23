#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
set -euo pipefail
IFS=$'\n\t'

target="$1"
shift

args=()
for arg in "$@"; do
    if [[ "${arg}" == *' '* ]] || [[ "${arg}" == *$'\t'* ]]; then
        args+=("'${arg}'")
    else
        args+=("${arg}")
    fi
done

qemu_user() {
    qemu_arch="$1"
    shift
    if which "qemu-${qemu_arch}" >/dev/null; then
        "qemu-${qemu_arch}" "$@" "${args[@]}"
    else
        "qemu-${qemu_arch}-static" "$@" "${args[@]}"
    fi
}

case "${target}" in
    # AArch64
    aarch64_be* | arm64_be*)
        qemu_user aarch64_be
        ;;
    aarch64* | arm64*)
        qemu_user aarch64
        ;;
    # Cortex-M
    thumbv6m-*)
        qemu_user arm -cpu cortex-m0
        ;;
    thumbv7m-*)
        qemu_user arm -cpu cortex-m3
        ;;
    thumbv7em-*)
        qemu_user arm -cpu cortex-m4
        ;;
    thumbv8m.base-*)
        # TODO: As of QEMU 7.2, QEMU doesn't support -cpu cortex-m23
        qemu_user arm -cpu cortex-m33
        ;;
    thumbv8m.main-*)
        qemu_user arm -cpu cortex-m33
        ;;
    # Cortex-A (AArch32)
    armv7a*)
        qemu_user arm -cpu cortex-a9
        ;;
    armebv7a*)
        qemu_user armeb -cpu cortex-a9
        ;;
    # Cortex-R (AArch32)
    armv7r*hf)
        qemu_user arm -cpu cortex-r5f
        ;;
    armebv7r*hf)
        qemu_user armeb -cpu cortex-r5f
        ;;
    armv7r*)
        qemu_user arm -cpu cortex-r5
        ;;
    armebv7r*)
        qemu_user armeb -cpu cortex-r5
        ;;
    # ARMv4T
    armv4t* | thumbv4t*)
        # qemu-system-arm -cpu help | grep -E '9.*t|sa1'
        # all passed:
        # - ti925t (ARM9TDMI)
        # exit-only passed:
        # - sa1110, sa1100 (StrongARM)
        # not worked: N/A
        # https://github.com/qemu/qemu/blob/74c581b6452394e591f13beba9fea2ec0688e2f5/target/arm/cpu_tcg.c#L913
        qemu_user arm -cpu ti925t
        ;;
    # ARMv5TE
    armv5te* | thumbv5te*)
        qemu_user arm -cpu arm926
        ;;
    # RISC-V
    riscv32*)
        qemu_user riscv32
        ;;
    riscv64*)
        qemu_user riscv64
        ;;
    # MIPS
    mips*) echo "QEMU doesn't support semihosting for MIPS (${target}) with user-mode" && exit 1 ;;
    *) echo "unrecognized target ${target}" && exit 1 ;;
esac
