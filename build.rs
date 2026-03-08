// SPDX-License-Identifier: Apache-2.0 OR MIT

// The rustc-cfg emitted by the build script are *not* public API.

#[path = "src/gen/build.rs"]
mod generated;
#[path = "version.rs"]
mod version;

use std::env;

use self::version::{Version, rustc_version};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/gen/build.rs");

    let target_arch = &*env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");

    let version = match rustc_version() {
        Some(version) => version,
        None => {
            if env::var_os("SEMIHOSTING_DENY_WARNINGS").is_some() {
                panic!("unable to determine rustc version")
            }
            println!(
                "cargo:warning={}: unable to determine rustc version; assuming latest stable rustc (1.{})",
                env!("CARGO_PKG_NAME"),
                Version::LATEST.minor
            );
            Version::LATEST
        }
    };

    if version.minor >= 80 {
        // Custom cfgs set by build script. Not public API.
        // grep -F 'cargo:rustc-cfg=' build.rs | grep -Ev '^ *//' | sed -E 's/^.*cargo:rustc-cfg=//; s/(=\\)?".*$//' | LC_ALL=C sort -u | tr '\n' ',' | sed -E 's/,$/\n/'
        println!(
            "cargo:rustc-check-cfg=cfg(semihosting_no_asm,semihosting_no_duration_checked_float,semihosting_no_error_in_core,semihosting_no_strict_provenance,semihosting_target_feature)"
        );
        // TODO: handle multi-line target_feature_fallback
        // grep -F 'target_feature_fallback("' build.rs | grep -Ev '^ *//' | sed -E 's/^.*target_feature_fallback\(//; s/",.*$/"/' | LC_ALL=C sort -u | tr '\n' ',' | sed -E 's/,$/\n/'
        println!(
            r#"cargo:rustc-check-cfg=cfg(semihosting_target_feature,values("mclass","thumb-mode"))"#
        );
    }

    // Note that cfgs are `no_`*, not `has_*`. This allows treating as the latest
    // stable rustc is used when the build script doesn't run. This is useful
    // for non-cargo build systems that don't run the build script.

    // duration_checked_float stabilized in Rust 1.66 (nightly-2022-10-25): https://github.com/rust-lang/rust/pull/102271
    if !version.probe(66, 2022, 10, 24) {
        println!("cargo:rustc-cfg=semihosting_no_duration_checked_float");
    }
    // error_in_core stabilized in Rust 1.81 (nightly-2024-06-09): https://github.com/rust-lang/rust/pull/125951
    if !version.probe(81, 2024, 6, 8) {
        println!("cargo:rustc-cfg=semihosting_no_error_in_core");
    }
    // strict_provenance/exposed_provenance APIs stabilized in Rust 1.84 (nightly-2024-10-22): https://github.com/rust-lang/rust/pull/130350
    if !version.probe(84, 2024, 10, 21) {
        println!("cargo:rustc-cfg=semihosting_no_strict_provenance");
    }

    match target_arch {
        "arm" => {
            let target = &*env::var("TARGET").expect("TARGET not set");

            // https://github.com/rust-lang/rust/pull/123745 (includes https://github.com/rust-lang/cargo/pull/13560) merged in Rust 1.79 (nightly-2024-04-11).
            if !version.probe(79, 2024, 4, 10) {
                // HACK: If --target is specified, rustflags is not applied to the build
                // script itself, so the build script will not be recompiled when rustflags
                // is changed. That in itself is not a problem, but the old Cargo does
                // not rerun the build script as well, which can be problematic.
                // https://github.com/rust-lang/cargo/issues/13003
                // This problem has been fixed in 1.79 so only older versions need a workaround.
                println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
                println!("cargo:rerun-if-env-changed=RUSTFLAGS");
                println!("cargo:rerun-if-env-changed=CARGO_BUILD_RUSTFLAGS");
                let mut target_upper = target.replace(['-', '.'], "_");
                target_upper.make_ascii_uppercase();
                println!("cargo:rerun-if-env-changed=CARGO_TARGET_{target_upper}_RUSTFLAGS");
            }

            if needs_target_feature_fallback(&version) {
                // See https://github.com/taiki-e/atomic-maybe-uninit/blob/HEAD/build.rs for details
                let mut subarch =
                    target.strip_prefix("arm").or_else(|| target.strip_prefix("thumb")).unwrap();
                subarch = subarch.strip_prefix("eb").unwrap_or(subarch); // ignore endianness
                subarch = subarch.split('-').next().unwrap(); // ignore vender/os/env
                subarch = subarch.split('.').next().unwrap(); // ignore .base/.main suffix
                let mut mclass = false;
                match subarch {
                    "v6m" | "v7em" | "v7m" | "v8m" => mclass = true,
                    _ => {}
                }
                target_feature_fallback("mclass", mclass);
                // All builtin targets that start with "thumb" enable thumb-mode, and
                // some builtin targets that start with "arm" are also enable thumb-mode.
                let thumb_mode =
                    target.starts_with("thumb") || generated::ARM_BUT_THUMB_MODE.contains(&target);
                target_feature_fallback("thumb-mode", thumb_mode);
            }
        }
        "loongarch32" => {
            // asm! on loongarch32 stabilized in Rust 1.91 (nightly-2025-08-11): https://github.com/rust-lang/rust/pull/144402
            if !version.probe(91, 2025, 8, 10) {
                println!("cargo:rustc-cfg=semihosting_no_asm");
            }
        }
        _ => {}
    }
}

// HACK: Currently, it seems that the only way to handle unstable target
// features on the stable is to parse the `-C target-feature` in RUSTFLAGS.
//
// - #[cfg(target_feature = "unstable_target_feature")] doesn't work on stable.
// - CARGO_CFG_TARGET_FEATURE excludes unstable target features on stable.
//
// As mentioned in the [RFC2045], unstable target features are also passed to LLVM
// (e.g., https://godbolt.org/z/4rr7rMcfG), so this hack works properly on stable.
//
// [RFC2045]: https://rust-lang.github.io/rfcs/2045-target-feature.html#backend-compilation-options
fn needs_target_feature_fallback(version: &Version) -> bool {
    if version.nightly {
        // In this case, cfg(target_feature = "...") would work, so skip emitting our own fallback target_feature cfg.
        false
    } else {
        true
    }
}
fn target_feature_fallback(name: &str, mut has_target_feature: bool) {
    if let Some(rustflags) = env::var_os("CARGO_ENCODED_RUSTFLAGS") {
        for mut flag in rustflags.to_string_lossy().split('\x1f') {
            flag = flag.strip_prefix("-C").unwrap_or(flag);
            if let Some(flag) = flag.strip_prefix("target-feature=") {
                for s in flag.split(',') {
                    // TODO: Handles cases where a specific target feature
                    // implicitly enables another target feature.
                    match (s.as_bytes().first(), s.as_bytes().get(1..)) {
                        (Some(b'+'), Some(f)) if f == name.as_bytes() => has_target_feature = true,
                        (Some(b'-'), Some(f)) if f == name.as_bytes() => has_target_feature = false,
                        _ => {}
                    }
                }
            }
        }
    }
    if has_target_feature {
        println!("cargo:rustc-cfg=semihosting_target_feature=\"{name}\"");
    }
}
