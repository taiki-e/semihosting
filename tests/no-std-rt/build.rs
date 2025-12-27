// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=raspi");
    println!("cargo:rustc-check-cfg=cfg(cortex_m_rt,aarch32_rt)");

    let target = &*env::var("TARGET").expect("TARGET not set");
    let target_arch = &*env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");

    let out_dir: PathBuf = env::var_os("OUT_DIR").expect("OUT_DIR not set").into();
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for e in fs::read_dir(manifest_dir).unwrap() {
        let path = e.unwrap().path();
        if path.extension().map_or(false, |e| e == "ld" || e == "x") {
            let path = path.strip_prefix(manifest_dir).unwrap();
            println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
        }
    }

    match target_arch {
        "aarch64" => {
            fs::write(out_dir.join("link.x"), include_bytes!("raspi/kernel.ld")).unwrap();
        }
        "arm" => {
            let mut subarch =
                target.strip_prefix("arm").or_else(|| target.strip_prefix("thumb")).unwrap();
            subarch = subarch.strip_prefix("eb").unwrap_or(subarch); // ignore endianness
            subarch = subarch.split('-').next().unwrap(); // ignore vender/os/env
            subarch = subarch.split('.').next().unwrap(); // ignore .base/.main suffix
            match subarch {
                "v6m" | "v7em" | "v7m" | "v8m" => {
                    println!("cargo:rustc-cfg=cortex_m_rt");
                    fs::write(out_dir.join("memory.x"), include_bytes!("arm-mclass-memory.x"))
                        .unwrap();
                }
                "v4t" | "v5te" | "v6" | "v7a" | "v7r" if cfg!(feature = "qemu-system") => {
                    println!("cargo:rustc-cfg=aarch32_rt");
                    fs::write(out_dir.join("memory.x"), include_bytes!("arm-versatileab-memory.x"))
                        .unwrap();
                }
                _ => {}
            }
        }
        "riscv32" => {
            fs::write(out_dir.join("riscv-common.ld"), include_bytes!("riscv-common.ld")).unwrap();
            fs::write(out_dir.join("link.x"), include_bytes!("riscv32.ld")).unwrap();
        }
        "riscv64" => {
            fs::write(out_dir.join("riscv-common.ld"), include_bytes!("riscv-common.ld")).unwrap();
            fs::write(out_dir.join("link.x"), include_bytes!("riscv64.ld")).unwrap();
        }
        _ if target_arch.starts_with("mips") => {
            fs::write(out_dir.join("link.x"), include_bytes!("mips.ld")).unwrap();
        }
        _ => {}
    }

    println!("cargo:rustc-link-search={}", out_dir.display());
}
