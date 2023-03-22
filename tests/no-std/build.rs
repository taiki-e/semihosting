// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=raspi");

    let host = &*env::var("HOST").expect("TARGET not set");
    let target = &*env::var("TARGET").expect("TARGET not set");
    let target_arch = &*env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for e in fs::read_dir(manifest_dir).unwrap() {
        let path = e.unwrap().path();
        if path.extension().map_or(false, |e| e == "ld" || e == "x") {
            let path = path.strip_prefix(manifest_dir).unwrap();
            println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
        }
    }

    if host.contains("-linux") {
        println!("cargo:rustc-cfg=host_linux");
    }
    if host.contains("-darwin") {
        println!("cargo:rustc-cfg=host_macos");
    }
    if host.contains("-windows") {
        println!("cargo:rustc-cfg=host_windows");
    }
    if target_arch == "arm" {
        let mut subarch =
            target.strip_prefix("arm").or_else(|| target.strip_prefix("thumb")).unwrap();
        subarch = subarch.strip_prefix("eb").unwrap_or(subarch); // ignore endianness
        subarch = subarch.split('-').next().unwrap(); // ignore vender/os/env
        subarch = subarch.split('.').next().unwrap(); // ignore .base/.main suffix
        match subarch {
            "v6m" | "v7em" | "v7m" | "v8m" => {
                println!("cargo:rustc-cfg=mclass");
                if subarch == "v8m" {
                    println!("cargo:rustc-cfg=thumbv8m");
                }
            }
            "v5te" => println!("cargo:rustc-cfg=armv5te"),
            "v4t" => println!("cargo:rustc-cfg=armv4t"),
            _ => {}
        }
    }
}
