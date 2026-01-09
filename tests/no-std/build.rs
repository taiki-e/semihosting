// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-check-cfg=cfg(mips,arm_compat)");
    println!(r#"cargo:rustc-check-cfg=cfg(host_os,values("windows"))"#);

    let target_arch = &*env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");

    if target_arch.starts_with("mips") {
        println!("cargo:rustc-cfg=mips");
    } else {
        println!("cargo:rustc-cfg=arm_compat");
    }

    let target = env::var("TARGET").expect("TARGET not set");
    let host = env::var("HOST").expect("HOST not set");
    let profile = env::var("PROFILE").expect("PROFILE not set");
    let out_dir: PathBuf = env::var_os("OUT_DIR").expect("OUT_DIR not set").into();
    let sep = if host.contains("-windows") {
        println!(r#"cargo:rustc-cfg=host_os="windows""#);
        '\\'
    } else {
        '/'
    };
    fs::write(
        out_dir.join("expected-bin-path"),
        format!(
            "const EXPECTED_BIN_PATH: &str = r\"{sep}{target}{sep}{profile}{sep}no-std-test\";\n"
        ),
    )
    .unwrap();
}
