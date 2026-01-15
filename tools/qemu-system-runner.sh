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
args=(
  -display none
  -kernel "${bin}"
)
semihosting_args=("$@")

semi_config='enable=on'
if [[ -n "${QEMU_SYSTEM_RUNNER_ARG_SPACES_SEPARATED:-}" ]]; then
  for arg in "${semihosting_args[@]}"; do
    if [[ "${arg}" == *' '* ]] || [[ "${arg}" == *$'\t'* ]]; then
      semi_config+=",arg='${arg}'"
    else
      semi_config+=",arg=${arg}"
    fi
  done
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
    semi_config+=",arg=${arg_string}"
  fi
fi
args+=(-semihosting-config "${semi_config}")

bin_dir=''
case "${target}" in
  aarch64_be*) bin_dir="${AARCH64_BE_QEMU_SYSTEM_BIN_DIR:+"${AARCH64_BE_QEMU_SYSTEM_BIN_DIR%/}/"}" ;;
  mips*) bin_dir="${MIPS_QEMU_SYSTEM_BIN_DIR:+"${MIPS_QEMU_SYSTEM_BIN_DIR%/}/"}" ;;
  loongarch*) bin_dir="${LOONGARCH_QEMU_BIN_DIR:+"${LOONGARCH_QEMU_BIN_DIR%/}/"}" ;;
esac
[[ -n "${bin_dir:-}" ]] || bin_dir="${QEMU_SYSTEM_BIN_DIR:+"${QEMU_SYSTEM_BIN_DIR%/}/"}"

qemu_system() {
  local qemu="${bin_dir}qemu-system-$1"
  shift
  if "${qemu}" --version | grep -Eq "QEMU emulator version ([8-9]|[1-9][0-9])\."; then
    args+=(-audio none)
  fi
  (
    set -x
    "${qemu}" --version
    "${qemu}" "$@" "${args[@]}"
  )
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
    # TODO: As of QEMU 10.2, QEMU doesn't support -cpu cortex-m23
    qemu_system arm -M lm3s6965evb -cpu cortex-m33
    ;;
  thumbv8m.main-*)
    qemu_system arm -M lm3s6965evb -cpu cortex-m33
    ;;
  # Cortex-A (AArch32)
  armv7a* | armebv7a* | thumbv7a*)
    qemu_system arm -M xilinx-zynq-a9 -cpu cortex-a9
    ;;
  # Cortex-R (AArch32)
  armv7r*hf | armebv7r*hf | thumbv7r*hf)
    qemu_system arm -M versatilepb -cpu cortex-r5f
    ;;
  armv7r* | armebv7r* | thumbv7r*)
    qemu_system arm -M versatilepb -cpu cortex-r5
    ;;
  armv8r* | armebv8r* | thumbv8r*)
    qemu_system arm -M mps3-an536 -cpu cortex-r52
    ;;
  # Armv4T
  armv4t* | thumbv4t*)
    qemu_system arm -M versatilepb -cpu ti925t
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
  # LoongArch
  loongarch32*)
    qemu_system loongarch64 -M virt -cpu la132
    ;;
  loongarch64*)
    qemu_system loongarch64 -M virt
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
