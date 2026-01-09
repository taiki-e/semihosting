// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=raspi");
    println!("cargo:rustc-check-cfg=cfg(cortex_m_rt,aarch32_rt)");
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for e in fs::read_dir(manifest_dir).unwrap() {
        let path = e.unwrap().path();
        if path.extension().map_or(false, |e| e == "ld" || e == "x") {
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
            subarch = subarch.split('-').next().unwrap(); // ignore vender/os/env
            subarch = subarch.split('.').next().unwrap(); // ignore .base/.main suffix
            match subarch {
                "v6m" | "v7em" | "v7m" | "v8m" => {
                    println!("cargo:rustc-cfg=cortex_m_rt");
                    fs::copy(manifest_dir.join("arm-mclass-memory.x"), out_dir.join("memory.x"))
                        .unwrap();
                    check_link("link.x");
                }
                "v4t" | "v5te" | "v6" | "v7r" | "v8r" => {
                    println!("cargo:rustc-cfg=aarch32_rt");
                    let memory_x = if subarch == "v8r" {
                        "arm-mps3-an536-memory.x"
                    } else {
                        "arm-versatileab-memory.x"
                    };
                    fs::copy(manifest_dir.join(memory_x), out_dir.join("memory.x")).unwrap();
                    check_link("link.x");
                }
                _ => {}
            }
        }
        "riscv32" | "riscv64" => {
            fs::write(out_dir.join("riscv-common.ld"), include_bytes!("riscv-common.ld")).unwrap();
            fs::copy(manifest_dir.join(format!("{target_arch}.ld")), out_dir.join("link.x"))
                .unwrap();
            check_link("link.x");
        }
        "mips" | "mips32r6" | "mips64" | "mips64r6" => {
            fs::copy(manifest_dir.join("mips.ld"), out_dir.join("link.x")).unwrap();
            check_link("link.x");
        }
        _ => {}
    }

    println!("cargo:rustc-link-search={}", out_dir.display());
}

fn check_link(expected_linker: &str) {
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
