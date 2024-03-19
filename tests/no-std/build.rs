// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let host = &*env::var("HOST").expect("TARGET not set");
    let target = &*env::var("TARGET").expect("TARGET not set");
    let target_arch = &*env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");

    if host.contains("-linux") {
        println!("cargo:rustc-cfg=host_linux");
    }
    if host.contains("-darwin") {
        println!("cargo:rustc-cfg=host_macos");
    }
    if host.contains("-windows") {
        println!("cargo:rustc-cfg=host_windows");
    }
    if target_arch.starts_with("mips") {
        println!("cargo:rustc-cfg=mips");
    } else {
        println!("cargo:rustc-cfg=arm_compat");
    }
    if target_arch == "arm" {
        let mut subarch =
            target.strip_prefix("arm").or_else(|| target.strip_prefix("thumb")).unwrap();
        subarch = subarch.strip_prefix("eb").unwrap_or(subarch); // ignore endianness
        subarch = subarch.split('-').next().unwrap(); // ignore vender/os/env
        subarch = subarch.split('.').next().unwrap(); // ignore .base/.main suffix
        match subarch {
            "v4t" => println!("cargo:rustc-cfg=armv4t"),
            _ => {}
        }
    }
}
