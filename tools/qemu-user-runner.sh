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
  if type -P "qemu-${qemu_arch}" >/dev/null; then
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
    # TODO: As of QEMU 9.2, QEMU doesn't support -cpu cortex-m23
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
  armv8r*)
    qemu_user arm -cpu cortex-r52
    ;;
  armebv8r*)
    qemu_user armeb -cpu cortex-r52
    ;;
  # Armv4T
  armv4t* | thumbv4t*)
    # qemu-system-arm -cpu help | grep -E '9.*t|sa1'
    # all passed:
    # - ti925t (ARM9TDMI)
    # exit-only passed:
    # - sa1110, sa1100 (StrongARM)
    # not worked: N/A
    # https://github.com/qemu/qemu/blob/v9.2.0/target/arm/tcg/cpu32.c#L778
    qemu_user arm -cpu ti925t
    ;;
  # Armv5TE
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
  mips*) bail "QEMU doesn't support semihosting for MIPS (${target}) with user-mode" ;;
  *) bail "unrecognized target ${target}" ;;
esac
