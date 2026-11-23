// SPDX-License-Identifier: Apache-2.0 OR MIT

// version.rs is shared with semihosting's build script, and no-std-rt only uses a part of it.
#[allow(dead_code)]
#[path = "src/build/version.rs"]
mod version;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use self::version::rustc_version;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=raspi");
    println!(
        "cargo:rustc-check-cfg=cfg(arm_m,arm_ar,arm_v6,arm_v8r,arm_hard_float,semihosting_no_asm)"
    );
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for e in fs::read_dir(manifest_dir).unwrap() {
        let path = e.unwrap().path();
        if path.extension().map_or(false, |e| e == "ld") {
            let path = path.strip_prefix(manifest_dir).unwrap();
            println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
        }
    }

    if cfg!(feature = "qemu-system") {
        setup_for_qemu_system(manifest_dir);
    }
}

fn setup_for_qemu_system(manifest_dir: &Path) {
    let target_arch = &*env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
    let out_dir: PathBuf = env::var_os("OUT_DIR").expect("OUT_DIR not set").into();

    match target_arch {
        "aarch64" => {
            fs::copy(manifest_dir.join("raspi/kernel.ld"), out_dir.join("link.x")).unwrap();
            check_link("link.x");
        }
        "arm" => {
            let target = &*env::var("TARGET").expect("TARGET not set");
            let mut subarch =
                target.strip_prefix("arm").or_else(|| target.strip_prefix("thumb")).unwrap();
            subarch = subarch.strip_prefix("eb").unwrap_or(subarch); // ignore endianness
            subarch = subarch.split_once('-').unwrap_or((subarch, "")).0; // ignore vendor/os/env
            let (subarch, _suffix) = subarch.rsplit_once('.').unwrap_or((subarch, "")); // .base/.main suffix
            if target.ends_with("hf") {
                println!("cargo:rustc-cfg=arm_hard_float");
            }
            match subarch {
                "v6m" | "v7em" | "v7m" | "v8m" | "v8.1m" => {
                    println!("cargo:rustc-cfg=arm_m");
                    fs::copy(manifest_dir.join("arm-m.ld"), out_dir.join("link.x")).unwrap();
                    check_link("link.x");
                }
                "v4t" | "v5te" | "v6" | "v7r" | "v8r" => {
                    println!("cargo:rustc-cfg=arm_ar");
                    if subarch == "v6" {
                        println!("cargo:rustc-cfg=arm_v6");
                    }
                    let (code_addr, code_end, ram_addr, ram_end) = if subarch == "v8r" {
                        println!("cargo:rustc-cfg=arm_v8r");
                        // MPS3-AN536: QSPI and DDR.
                        (0x0800_0000, 0x0880_0000, 0x2000_0000, 0x2800_0000)
                    } else {
                        // VersatilePB: SDRAM.
                        (0, 0x0800_0000, 0, 0x0800_0000)
                    };
                    fs::copy(manifest_dir.join("arm-ar.ld"), out_dir.join("arm-ar.ld")).unwrap();
                    fs::write(
                        out_dir.join("link.x"),
                        format!(
                            "\
                        CODE_ADDR = {code_addr:#x};\n\
                        CODE_END = {code_end:#x};\n\
                        RAM_ADDR = {ram_addr:#x};\n\
                        RAM_END = {ram_end:#x};\n\
                        INCLUDE arm-ar.ld\n\
                        "
                        ),
                    )
                    .unwrap();
                    check_link("link.x");
                }
                _ => {}
            }
        }
        _ => {
            let is_riscv = target_arch.starts_with("riscv");
            let is_loongarch = target_arch.starts_with("loongarch");
            let is_mips = target_arch.starts_with("mips");
            if is_riscv || is_loongarch || is_mips {
                let fw_jump_addr = match target_arch {
                    // https://github.com/riscv-software-src/opensbi/blob/v1.8/platform/template/objects.mk#L72
                    "riscv32" => "0x80400000",
                    "riscv64" => "0x80200000",
                    // https://github.com/qemu/qemu/blob/v10.2.0/tests/tcg/loongarch64/system/kernel.ld#L5
                    _ if is_loongarch => "0x200000",
                    "mips" | "mips32r6" => "0x80080000",
                    "mips64" | "mips64r6" => "0xffffffff80080000",
                    _ => unreachable!(),
                };
                fs::copy(
                    manifest_dir.join("riscv-loongarch-mips-common.ld"),
                    out_dir.join("common.ld"),
                )
                .unwrap();
                fs::write(
                    out_dir.join("link.x"),
                    format!(
                        "\
                    FW_JUMP_ADDR = {fw_jump_addr};\n\
                    INCLUDE common.ld\n\
                    "
                    ),
                )
                .unwrap();
                check_link("link.x");
            }
            // asm! on loongarch32 stabilized in Rust 1.91 (nightly-2025-08-11): https://github.com/rust-lang/rust/pull/144402
            if target_arch == "loongarch32" && !rustc_version().unwrap().probe(91, 2025, 8, 10) {
                println!("cargo:rustc-cfg=semihosting_no_asm");
            }
        }
    }

    println!("cargo:rustc-link-search={}", out_dir.display());
}

fn check_link(expected_linker: &str) {
    if cfg!(feature = "disable-link-check") {
        return;
    }
    if let Some(rustflags) = env::var_os("CARGO_ENCODED_RUSTFLAGS") {
        for mut flag in rustflags.to_string_lossy().split('\x1f') {
            flag = flag.strip_prefix("-C").unwrap_or(flag);
            if let Some(linker) = flag.strip_prefix("link-arg=-T") {
                if linker == expected_linker {
                    return;
                }
            }
        }
    }
    panic!("missing `-C link-arg=-T{expected_linker}` in rustflags")
}
