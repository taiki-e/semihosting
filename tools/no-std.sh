#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
set -CeEuo pipefail
IFS=$'\n\t'
trap -- 's=$?; printf >&2 "%s\n" "${0##*/}:${LINENO}: \`${BASH_COMMAND}\` exit with ${s}"; exit ${s}' ERR
trap -- 'printf >&2 "%s\n" "${0##*/}: trapped SIGINT"; exit 1' SIGINT
cd -- "$(dirname -- "$0")"/..

# USAGE:
#    ./tools/no-std.sh [+toolchain] [target]...
#    TEST_RUNNER=qemu-user ./tools/no-std.sh [+toolchain] [target]...

# rustc -Z unstable-options --print all-target-specs-json | jq -r '. | to_entries[] | if .value.os then empty else .key end'
default_targets=(
  # aarch64
  aarch64-unknown-none
  aarch64-unknown-none-softfloat
  aarch64_be-unknown-none-softfloat

  # arm
  # v4T
  armv4t-none-eabi
  thumbv4t-none-eabi
  # v5TE
  armv5te-none-eabi
  thumbv5te-none-eabi
  # v6
  armv6-none-eabi
  armv6-none-eabihf
  thumb6-none-eabi
  # v7-A
  armv7a-none-eabi
  armv7a-none-eabihf
  # v7-R
  armv7r-none-eabi
  armv7r-none-eabihf
  armebv7r-none-eabi
  armebv7r-none-eabihf
  # v8-R
  armv8r-none-eabihf
  armebv8r-none-eabihf # custom target
  # v6-M
  thumbv6m-none-eabi
  # v7-M
  thumbv7m-none-eabi
  thumbv7em-none-eabi
  thumbv7em-none-eabihf
  # v8-M
  thumbv8m.base-none-eabi
  thumbv8m.main-none-eabi
  thumbv8m.main-none-eabihf

  # riscv32
  riscv32i-unknown-none-elf
  riscv32im-unknown-none-elf
  riscv32imc-unknown-none-elf
  riscv32ima-unknown-none-elf
  riscv32imac-unknown-none-elf
  riscv32imafc-unknown-none-elf
  riscv32gc-unknown-none-elf # custom target
  riscv32e-unknown-none-elf
  riscv32em-unknown-none-elf
  riscv32emc-unknown-none-elf
  # riscv64
  riscv64im-unknown-none-elf
  riscv64imac-unknown-none-elf
  riscv64gc-unknown-none-elf

  # mips32r2
  mips-unknown-none # custom target
  mipsel-unknown-none
  # mips32r6
  mipsisa32r6-unknown-none   # custom target
  mipsisa32r6el-unknown-none # custom target
  # mips64r2
  mips64-unknown-none   # custom target
  mips64el-unknown-none # custom target
  # mips64r6
  mipsisa64r6-unknown-none   # custom target
  mipsisa64r6el-unknown-none # custom target
)

x() {
  (
    set -x
    "$@"
  )
}
x_cargo() {
  if [[ -n "${RUSTFLAGS:-}" ]]; then
    printf '%s\n' "+ RUSTFLAGS='${RUSTFLAGS}' \\"
  fi
  x cargo "$@"
  printf '\n'
}
retry() {
  for i in {1..10}; do
    if "$@"; then
      return 0
    else
      sleep "${i}"
    fi
  done
  "$@"
}
bail() {
  printf >&2 'error: %s\n' "$*"
  exit 1
}
info() {
  printf >&2 'info: %s\n' "$*"
}

pre_args=()
is_custom_toolchain=''
if [[ "${1:-}" == "+"* ]]; then
  if [[ "$1" == "+esp" ]]; then
    # shellcheck disable=SC1091
    . "${HOME}/export-esp.sh"
    is_custom_toolchain=1
  fi
  pre_args+=("$1")
  shift
fi
if [[ $# -gt 0 ]]; then
  targets=("$@")
else
  targets=("${default_targets[@]}")
fi
runner="${TEST_RUNNER:-qemu-system}"

rustup_target_list=''
if [[ -z "${is_custom_toolchain}" ]]; then
  rustup_target_list=$(rustup ${pre_args[@]+"${pre_args[@]}"} target list | cut -d' ' -f1)
fi
rustc_target_list=$(rustc ${pre_args[@]+"${pre_args[@]}"} --print target-list)
rustc_version=$(rustc ${pre_args[@]+"${pre_args[@]}"} -vV | grep -E '^release:' | cut -d' ' -f2)
llvm_version=$(rustc ${pre_args[@]+"${pre_args[@]}"} -vV | { grep -E '^LLVM version:' || true; } | cut -d' ' -f3)
llvm_version="${llvm_version%%.*}"
target_dir=$(pwd)/target
nightly=''
if [[ "${rustc_version}" =~ nightly|dev ]]; then
  nightly=1
  if [[ -z "${is_custom_toolchain}" ]]; then
    retry rustup ${pre_args[@]+"${pre_args[@]}"} component add rust-src &>/dev/null
  fi
fi
workspace_root=$(pwd)
export SEMIHOSTING_DENY_WARNINGS=1

run() {
  local target="$1"
  shift
  local target_lower="${target//-/_}"
  local target_lower="${target_lower//./_}"
  local target_upper
  target_upper=$(tr '[:lower:]' '[:upper:]' <<<"${target_lower}")
  local args=(${pre_args[@]+"${pre_args[@]}"})
  local target_rustflags="${RUSTFLAGS:-}"
  if ! grep -Eq "^${target}$" <<<"${rustc_target_list}" || [[ -f "target-specs/${target}.json" ]]; then
    if [[ "${target}" == "riscv64im-unknown-none-elf" ]]; then
      target=riscv64i-unknown-none-elf # custom target
    fi
    if [[ ! -f "target-specs/${target}.json" ]]; then
      info "target '${target}' not available on ${rustc_version} (skipped)"
      return 0
    fi
    local target_flags=(--target "$(pwd)/target-specs/${target}.json")
  else
    local target_flags=(--target "${target}")
  fi
  local subcmd=run
  args+=("${subcmd}" "${target_flags[@]}")
  build_std=()
  if grep -Eq "^${target}$" <<<"${rustup_target_list}"; then
    retry rustup ${pre_args[@]+"${pre_args[@]}"} target add "${target}" &>/dev/null
  elif [[ -n "${nightly}" ]]; then
    build_std=(-Z build-std="core")
  else
    info "target '${target}' requires nightly compiler (skipped)"
    return 0
  fi
  if [[ "${target_rustflags}" == *"panic=unwind"* ]]; then
    build_std=(-Z build-std="core,alloc")
    args+=(--features panic-unwind)
  elif [[ "${target_rustflags}" == *"force-unwind-tables"* ]]; then
    build_std=(-Z build-std="core")
  fi

  local test_dir=tests/no-std
  case "${target}" in
    aarch64_be*)
      case "${runner}" in
        qemu-system)
          # TODO: QEMU exit with 1
          info "QEMU bug on aarch64_be (${target}) with system-mode (skipped)"
          return 0
          ;;
      esac
      ;;
    aarch64* | arm64* | riscv*)
      case "${runner}" in
        qemu-system)
          linker=link.x
          target_rustflags+=" -C link-arg=-T${linker}"
          ;;
      esac
      ;;
    armebv7r*)
      if [[ "${llvm_version}" -lt 17 ]]; then
        # pre-17 LLD doesn't support big-endian Arm
        target_rustflags+=" -C linker=arm-none-eabi-ld -C link-arg=-EB"
      fi
      ;;
    thumbv6m* | thumbv7m* | thumbv7em* | thumbv8m*)
      case "${runner}" in
        qemu-system)
          linker=link.x
          target_rustflags+=" -C link-arg=-T${linker}"
          ;;
        # TODO: qemu-arm: ../../accel/tcg/translate-all.c:1381: page_set_flags: Assertion `end - 1 <= GUEST_ADDR_MAX' failed.
        qemu-user)
          info "QEMU doesn't support Cortex-M (${target}) with user-mode (skipped)"
          return 0
          ;;
      esac
      ;;
    armv[456]* | thumbv[456]*)
      case "${runner}" in
        qemu-system)
          linker=link.x
          target_rustflags+=" -C link-arg=-T${linker}"
          ;;
      esac
      ;;
    mips*)
      case "${runner}" in
        qemu-system)
          # On QEMU 8.0+, QEMU doesn't seem to support semihosting for MIPS. https://qemu-project.gitlab.io/qemu/about/removed-features.html#mips-trap-and-emulate-kvm-support-removed-in-8-0
          if qemu-system-mips --version | grep -Eq "QEMU emulator version ([8-9]|[1-9][0-9])\."; then
            info "QEMU doesn't support semihosting for MIPS (${target}) on QEMU 8.0+ (skipped)"
            return 0
          fi
          linker=link.x
          # Allow linker_messages to work around https://github.com/llvm/llvm-project/issues/56192.
          target_rustflags+=" -C link-arg=-T${linker} -A linker_messages"
          ;;
        # As of QEMU 7.2, QEMU doesn't support semihosting for MIPS with user-mode.
        # https://www.qemu.org/docs/master/about/emulation.html#supported-targets
        qemu-user)
          info "QEMU doesn't support semihosting for MIPS (${target}) with user-mode (skipped)"
          return 0
          ;;
      esac
      ;;
  esac

  args+=(--features "${runner}")
  (
    cd -- "${test_dir}"
    case "$(uname -s)" in
      MINGW* | MSYS* | CYGWIN* | Windows_NT) export "CARGO_TARGET_${target_upper}_RUNNER"="bash ${workspace_root}/tools/${runner}-runner.sh ${target}" ;;
      *) export "CARGO_TARGET_${target_upper}_RUNNER"="${workspace_root}/tools/${runner}-runner.sh ${target}" ;;
    esac
    test_args=(a '' "c d")

    RUSTFLAGS="${target_rustflags}" \
      x_cargo "${args[@]}" ${build_std[@]+"${build_std[@]}"} "$@" -- "${test_args[@]}" <<<"stdin"
    RUSTFLAGS="${target_rustflags}" \
      x_cargo "${args[@]}" ${build_std[@]+"${build_std[@]}"} --release "$@" -- "${test_args[@]}" <<<"stdin"

    QEMU_SYSTEM_RUNNER_ARG_SPACES_SEPARATED=1 \
      RUSTFLAGS="${target_rustflags}" \
      x_cargo "${args[@]}" ${build_std[@]+"${build_std[@]}"} "$@" -- "${test_args[@]}" <<<"stdin"
    QEMU_SYSTEM_RUNNER_ARG_SPACES_SEPARATED=1 \
      RUSTFLAGS="${target_rustflags}" \
      x_cargo "${args[@]}" ${build_std[@]+"${build_std[@]}"} --release "$@" -- "${test_args[@]}" <<<"stdin"

    case "${target}" in
      arm64* | thumbv6m* | thumbv7m* | thumbv7em* | thumbv8m*) ;;
      arm* | thumb*)
        RUSTFLAGS="${target_rustflags}" \
          x_cargo "${args[@]}" ${build_std[@]+"${build_std[@]}"} --features semihosting/trap-hlt "$@" -- "${test_args[@]}" <<<"stdin"
        RUSTFLAGS="${target_rustflags}" \
          x_cargo "${args[@]}" ${build_std[@]+"${build_std[@]}"} --features semihosting/trap-hlt --release "$@" -- "${test_args[@]}" <<<"stdin"
        ;;
    esac

    if [[ -n "${nightly}" ]]; then
      case "${runner}" in
        # TODO: Skip user mode due to "rust-lld: error: undefined symbol: __eh_frame".
        qemu-system)
          case "${target}" in
            riscv??e* | riscv32imafc-*) ;; # TODO: not yet supported by unwinding
            aarch64* | arm64* | riscv*)
              # Handle targets without atomic CAS
              case "${target}" in
                thumbv[4-5]t* | armv[4-5]t* | thumbv6m*)
                  args+=(--features portable-atomic)
                  target_rustflags+=" --cfg portable_atomic_unsafe_assume_single_core"
                  ;;
                riscv??[ie]-* | riscv??[ie]m-* | riscv??[ie]mc-*)
                  args+=(--features portable-atomic)
                  target_rustflags+=" --cfg portable_atomic_unsafe_assume_single_core --cfg portable_atomic_s_mode"
                  ;;
              esac
              # skip if we already tested with panic=unwind
              if [[ "${target_rustflags}" != *"panic=unwind"* ]]; then
                build_std=(-Z build-std="core,alloc")
                args+=(--features panic-unwind)
                target_rustflags+=" -C panic=unwind"
                CARGO_TARGET_DIR="${target_dir}/panic-unwind" \
                  RUSTFLAGS="${target_rustflags}" \
                  x_cargo "${args[@]}" ${build_std[@]+"${build_std[@]}"} "$@" -- "${test_args[@]}" <<<"stdin"
                CARGO_TARGET_DIR="${target_dir}/panic-unwind" \
                  RUSTFLAGS="${target_rustflags}" \
                  x_cargo "${args[@]}" ${build_std[@]+"${build_std[@]}"} --release "$@" -- "${test_args[@]}" <<<"stdin"
              fi
              ;;
          esac
          ;;
      esac
    fi
  )
}

for target in "${targets[@]}"; do
  run "${target}"
done
