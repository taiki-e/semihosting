#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
set -euo pipefail
IFS=$'\n\t'

target="$1"
shift

bin="$1"
args=(-display none -kernel "${bin}")
semihosting_args=("$@")

if [[ -n "${QEMU_SYSTEM_RUNNER_ARG_SPACES_SEPARATED:-}" ]]; then
    semi_config=''
    for arg in "${semihosting_args[@]}"; do
        if [[ -n "${semi_config}" ]]; then
            semi_config+=','
        fi
        if [[ "${arg}" == *' '* ]] || [[ "${arg}" == *$'\t'* ]]; then
            semi_config+="arg='${arg}'"
        else
            semi_config+="arg=${arg}"
        fi
    done
    if [[ -n "${semi_config}" ]]; then
        args+=(-semihosting-config "${semi_config}")
    else
        args+=(-semihosting)
    fi
else
    arg_string=''
    for arg in "${semihosting_args[@]}"; do
        if [[ "${arg}" != "${bin}" ]]; then
            arg_string+=' '
        fi
        if [[ "${arg}" == *' '* ]] || [[ "${arg}" == *$'\t'* ]]; then
            arg_string+="'${arg}'"
        else
            arg_string+="${arg}"
        fi
    done
    if [[ -n "${arg_string}" ]]; then
        args+=(-semihosting-config "arg=${arg_string}")
    else
        args+=(-semihosting)
    fi
fi

qemu_system() {
    qemu_arch="$1"
    shift

    "qemu-system-${qemu_arch}" "$@" "${args[@]}"
}

case "${target}" in
    # AArch64
    aarch64* | arm64*)
        qemu_system aarch64 -M raspi3b
        ;;
    # Cortex-M
    thumbv6m-*)
        qemu_system arm -cpu cortex-m0 -M lm3s6965evb
        ;;
    thumbv7m-*)
        qemu_system arm -cpu cortex-m3 -M lm3s6965evb
        ;;
    thumbv7em-*)
        qemu_system arm -cpu cortex-m4 -M lm3s6965evb
        ;;
    thumbv8m.base-*)
        # TODO: As of QEMU 7.2, QEMU doesn't support -cpu cortex-m23
        qemu_system arm -cpu cortex-m33 -M lm3s6965evb
        ;;
    thumbv8m.main-*)
        qemu_system arm -cpu cortex-m33 -M lm3s6965evb
        ;;
    # Cortex-A (AArch32)
    armv7a* | armebv7a*)
        qemu_system arm -M xilinx-zynq-a9
        ;;
    # Cortex-R (AArch32)
    armv7r* | armebv7r*)
        qemu_system arm -M xilinx-zynq-a9
        ;;
    # ARMv4T
    armv4t* | thumbv4t*)
        # qemu-system-arm -M help | grep -E '9.*T|SA-'
        # all passed: N/A
        # exit-only passed:
        # - collie (SA-1110)
        # not worked: N/A
        qemu_system arm -M collie
        ;;
    # ARMv5TE
    armv5te* | thumbv5te*)
        # qemu-system-arm -M help | grep -E 'ARM9|ARM10|PXA'
        # all passed: N/A
        # exit-only passed:
        # - integratorcp, musicpal, realview-eb, versatileab, versatilepb (ARM926EJ-S)
        # - tosa (PXA255)
        # - akita, borzoi, spitz, terrier (PXA270)
        # - mainstone, z2 (PXA27x)
        # not worked:
        # - canon-a1100 (ARM946)
        # - imx25-pdk (ARM926)
        # - palmetto-bmc, quanta-q71l-bmc, supermicrox11-bmc (ARM926EJ-S)
        # - connex (PXA255)
        # - verdex (PXA270)
        qemu_system arm -M integratorcp
        ;;
    # RISC-V
    riscv32*)
        qemu_system riscv32 -M virt
        ;;
    riscv64*)
        qemu_system riscv64 -M virt
        ;;
    # MIPS
    mips-*)
        qemu_system mips -M malta
        ;;
    mipsel-*)
        qemu_system mipsel -M malta
        ;;
    mips64-*)
        qemu_system mips64 -cpu MIPS64R2-generic -M malta
        ;;
    mips64el-*)
        qemu_system mips64el -cpu MIPS64R2-generic -M malta
        ;;
    *) echo "unrecognized target ${target}" && exit 1 ;;
esac
