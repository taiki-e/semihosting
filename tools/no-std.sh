#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
set -eEuo pipefail
IFS=$'\n\t'
cd "$(dirname "$0")"/..

# shellcheck disable=SC2154
trap 's=$?; echo >&2 "$0: error on line "${LINENO}": ${BASH_COMMAND}"; exit ${s}' ERR
trap -- 'echo >&2 "$0: trapped SIGINT"; exit 1' SIGINT

# USAGE:
#    ./tools/no-std.sh [+toolchain] [target]...

# rustc --print target-list | grep -E -e '-none|avr-'
# rustup target list | grep -E -e '-none|avr-'
default_targets=(
    # aarch64
    aarch64-unknown-none
    aarch64-unknown-none-softfloat

    # armv4t
    armv4t-none-eabi
    thumbv4t-none-eabi
    # armv5te
    armv5te-none-eabi
    thumbv5te-none-eabi

    # armv7-a
    armv7a-none-eabi
    armv7a-none-eabihf
    # armv7-r
    armv7r-none-eabi
    armv7r-none-eabihf
    armebv7r-none-eabi
    armebv7r-none-eabihf

    # armv6-m
    thumbv6m-none-eabi
    # armv7-m
    thumbv7m-none-eabi
    thumbv7em-none-eabi
    thumbv7em-none-eabihf
    # armv8-m
    thumbv8m.base-none-eabi
    thumbv8m.main-none-eabi
    thumbv8m.main-none-eabihf

    # riscv32
    riscv32i-unknown-none-elf
    riscv32im-unknown-none-elf
    riscv32imc-unknown-none-elf
    riscv32imac-unknown-none-elf
    riscv32gc-unknown-none-elf # custom target
    # riscv64
    riscv64i-unknown-none-elf # custom target
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
    local cmd="$1"
    shift
    (
        set -x
        "${cmd}" "$@"
    )
}
x_cargo() {
    if [[ -n "${RUSTFLAGS:-}" ]]; then
        echo "+ RUSTFLAGS='${RUSTFLAGS}' \\"
    fi
    RUSTFLAGS="${RUSTFLAGS:-}" \
        x cargo "$@"
    echo
}
bail() {
    echo >&2 "error: $*"
    exit 1
}
info() {
    echo >&2 "info: $*"
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
    rustup_target_list=$(rustup ${pre_args[@]+"${pre_args[@]}"} target list | sed 's/ .*//g')
fi
rustc_target_list=$(rustc ${pre_args[@]+"${pre_args[@]}"} --print target-list)
rustc_version=$(rustc ${pre_args[@]+"${pre_args[@]}"} -Vv | grep 'release: ' | sed 's/release: //')
nightly=''
if [[ "${rustc_version}" == *"nightly"* ]] || [[ "${rustc_version}" == *"dev"* ]]; then
    nightly=1
    if [[ -z "${is_custom_toolchain}" ]]; then
        rustup ${pre_args[@]+"${pre_args[@]}"} component add rust-src &>/dev/null
    fi
fi
workspace_root=$(pwd)

run() {
    local target="$1"
    shift
    local target_lower="${target//-/_}"
    local target_lower="${target_lower//./_}"
    local target_upper
    target_upper=$(tr '[:lower:]' '[:upper:]' <<<"${target_lower}")
    local args=(${pre_args[@]+"${pre_args[@]}"})
    local target_rustflags="${RUSTFLAGS:-}"
    if ! grep <<<"${rustc_target_list}" -Eq "^${target}$" || [[ -f "target-specs/${target}.json" ]]; then
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
    if grep <<<"${rustup_target_list}" -Eq "^${target}$"; then
        rustup ${pre_args[@]+"${pre_args[@]}"} target add "${target}" &>/dev/null
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
        armv4t* | thumbv4t*)
            if [[ -n "${CI:-}" ]] && [[ "${runner}" == "qemu-system" ]] && [[ "${OSTYPE}" == "linux"* ]]; then
                # Old QEMU we used in CI doesn't work on this case
                return 0
            fi
            ;;
        armebv7r*)
            # lld doesn't support big-endian arm
            target_rustflags+=" -C linker=arm-none-eabi-ld -C link-arg=-EB"
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
        aarch64* | arm64* | riscv*)
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
                    # On QEMU 8.0+, QEMU doesn't seem to support semihosting for MIPS.
                    if qemu-system-mips --version | grep -Eq "QEMU emulator version 8\."; then
                        info "QEMU doesn't support semihosting for MIPS (${target}) on QEMU 8.0+ (skipped)"
                        return 0
                    fi
                    linker=link.x
                    target_rustflags+=" -C link-arg=-T${linker}"
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
        cd "${test_dir}"
        case "${OSTYPE}" in
            cygwin* | msys*) export "CARGO_TARGET_${target_upper}_RUNNER"="bash ${workspace_root}/tools/${runner}-runner.sh ${target}" ;;
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

        if [[ -n "${nightly}" ]]; then
            case "${runner}" in
                qemu-system)
                    case "${target}" in
                        aarch64* | arm64* | riscv*)
                            # Handle targets without atomic CAS
                            case "${target}" in
                                thumbv[4-5]t* | armv[4-5]t* | thumbv6m*)
                                    args+=(--features portable-atomic)
                                    target_rustflags+=" --cfg portable_atomic_unsafe_assume_single_core"
                                    ;;
                                riscv??i-* | riscv??im-* | riscv??imc-*)
                                    args+=(--features portable-atomic)
                                    target_rustflags+=" --cfg portable_atomic_unsafe_assume_single_core --cfg portable_atomic_s_mode"
                                    ;;
                            esac
                            # skip if we already tested with panic=unwind
                            if [[ "${target_rustflags}" != *"panic=unwind"* ]]; then
                                build_std=(-Z build-std="core,alloc")
                                args+=(--features panic-unwind)
                                target_rustflags+=" -C panic=unwind"
                                CARGO_TARGET_DIR="../../target/panic-unwind" \
                                    RUSTFLAGS="${target_rustflags}" \
                                    x_cargo "${args[@]}" ${build_std[@]+"${build_std[@]}"} "$@" -- "${test_args[@]}" <<<"stdin"
                                CARGO_TARGET_DIR="../../target/panic-unwind" \
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
