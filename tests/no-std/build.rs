// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{env, fs, path::PathBuf, time::SystemTime};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-check-cfg=cfg(mips,arm_compat)");

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
    let sep = if host.contains("-windows") { '\\' } else { '/' };
    let duration_since_unix_epoch =
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    fs::write(
        out_dir.join("expected"),
        format!(
            "\
            const EXPECTED_BIN_PATH: &str = r\"{sep}{target}{sep}{profile}{sep}no-std-test\";\n\
            #[cfg(not(mips))]
            const EXPECTED_DURATION_SINCE_UNIX_EPOCH: Duration = Duration::from_secs({duration_since_unix_epoch});\n\
            "
        ),
    )
    .unwrap();
}
