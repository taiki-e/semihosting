#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
set -CeEuo pipefail
IFS=$'\n\t'
trap -- 's=$?; printf >&2 "%s\n" "${0##*/}:${LINENO}: \`${BASH_COMMAND}\` exit with ${s}"; exit ${s}' ERR

bail() {
  printf >&2 'error: %s\n' "$*"
  exit 1
}

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

export QEMU_AUDIO_DRV=none

case "${target}" in
  # AArch64
  aarch64* | arm64*)
    qemu_system aarch64 -M raspi3b
    ;;
  # Cortex-M
  thumbv6m-*)
    qemu_system arm -M lm3s6965evb -cpu cortex-m0
    ;;
  thumbv7m-*)
    qemu_system arm -M lm3s6965evb -cpu cortex-m3
    ;;
  thumbv7em-*)
    qemu_system arm -M lm3s6965evb -cpu cortex-m4
    ;;
  thumbv8m.base-*)
    # TODO: As of QEMU 10.1, QEMU doesn't support -cpu cortex-m23
    qemu_system arm -M lm3s6965evb -cpu cortex-m33
    ;;
  thumbv8m.main-*)
    qemu_system arm -M lm3s6965evb -cpu cortex-m33
    ;;
  # Cortex-A (AArch32)
  armv7a* | armebv7a*)
    qemu_system arm -M xilinx-zynq-a9
    ;;
  # Cortex-R (AArch32)
  armv7r* | armebv7r*)
    # TODO: As of QEMU 8.2, qemu-system-arm doesn't support Cortex-R machine.
    # TODO: mps3-an536 added in QEMU 9.0 is Cortex-R52 board (Armv8-R AArch32)
    qemu_system arm -M xilinx-zynq-a9
    ;;
  armv8r* | armebv8r*)
    # TODO: As of QEMU 8.2, qemu-system-arm doesn't support Cortex-R machine.
    # TODO: mps3-an536 added in QEMU 9.0 is Cortex-R52 board (Armv8-R AArch32)
    qemu_system arm -M xilinx-zynq-a9
    ;;
  # Armv4T
  armv4t* | thumbv4t*)
    # qemu-system-arm -M help | grep -E '9.*T|SA-|OMAP310'
    # all passed: N/A # TODO
    # exit-only passed:
    # - sx1, sx1-v1 (OMAP310)
    # - collie (SA-1110)
    # not worked: N/A
    qemu_system arm -M sx1
    ;;
  # Armv5TE
  armv5te* | thumbv5te*)
    qemu_system arm -M versatilepb -cpu arm926
    ;;
  # Armv6
  armv6* | thumbv6*)
    qemu_system arm -M versatilepb -cpu arm1176
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
  mipsisa32r6-*)
    qemu_system mips -M malta -cpu mips32r6-generic
    ;;
  mipsisa32r6el-*)
    qemu_system mipsel -M malta -cpu mips32r6-generic
    ;;
  mips64-*)
    qemu_system mips64 -M malta -cpu MIPS64R2-generic
    ;;
  mips64el-*)
    qemu_system mips64el -M malta -cpu MIPS64R2-generic
    ;;
  mipsisa64r6-*)
    qemu_system mips64 -M malta -cpu I6400
    ;;
  mipsisa64r6el-*)
    qemu_system mips64el -M malta -cpu I6400
    ;;
  *) bail "unrecognized target ${target}" ;;
esac
