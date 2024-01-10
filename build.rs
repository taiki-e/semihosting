// SPDX-License-Identifier: Apache-2.0 OR MIT

// The rustc-cfg emitted by the build script are *not* public API.

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_arch = &*env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
    if target_arch == "arm" {
        let target = &*env::var("TARGET").expect("TARGET not set");
        // HACK: If --target is specified, rustflags is not applied to the build
        // script itself, so the build script will not be rerun when these are changed.
        //
        // Ideally, the build script should be rebuilt when CARGO_ENCODED_RUSTFLAGS
        // is changed, but since it is an environment variable set by cargo,
        // as of 1.62.0-nightly, specifying it as rerun-if-env-changed does not work.
        println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
        println!("cargo:rerun-if-env-changed=RUSTFLAGS");
        println!("cargo:rerun-if-env-changed=CARGO_BUILD_RUSTFLAGS");
        let mut target_upper = target.replace(['-', '.'], "_");
        target_upper.make_ascii_uppercase();
        println!("cargo:rerun-if-env-changed=CARGO_TARGET_{target_upper}_RUSTFLAGS");

        let version = match rustc_version() {
            Some(version) => version,
            None => {
                println!(
                    "cargo:warning={}: unable to determine rustc version; assuming latest stable rustc",
                    env!("CARGO_PKG_NAME"),
                );
                Version::LATEST
            }
        };

        if target.starts_with("thumb") {
            target_feature_if("thumb-mode", true, &version);
        }
        // See portable-atomic and atomic-maybe-uninit's build.rs for more
        let mut subarch =
            target.strip_prefix("arm").or_else(|| target.strip_prefix("thumb")).unwrap();
        subarch = subarch.strip_prefix("eb").unwrap_or(subarch); // ignore endianness
        subarch = subarch.split('-').next().unwrap(); // ignore vender/os/env
        subarch = subarch.split('.').next().unwrap(); // ignore .base/.main suffix
        let mut is_mclass = false;
        match subarch {
            "v6m" | "v7em" | "v7m" | "v8m" => is_mclass = true,
            _ => {}
        }
        target_feature_if("mclass", is_mclass, &version);
    }
}

fn target_feature_if(name: &str, mut has_target_feature: bool, version: &Version) {
    // HACK: Currently, it seems that the only way to handle unstable target
    // features on the stable is to parse the `-C target-feature` in RUSTFLAGS.
    //
    // - #[cfg(target_feature = "unstable_target_feature")] doesn't work on stable.
    // - CARGO_CFG_TARGET_FEATURE excludes unstable target features on stable.
    //
    // As mentioned in the [RFC2045], unstable target features are also passed to LLVM
    // (e.g., https://godbolt.org/z/TfaEx95jc), so this hack works properly on stable.
    //
    // [RFC2045]: https://rust-lang.github.io/rfcs/2045-target-feature.html#backend-compilation-options
    if version.nightly {
        // In this case, cfg(target_feature = "...") would work, so skip emitting our own target_feature cfg.
        return;
    }
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

mod version {
    use std::{env, process::Command, str};

    pub(crate) fn rustc_version() -> Option<Version> {
        let rustc = env::var_os("RUSTC")?;
        // Use verbose version output because the packagers add extra strings to the normal version output.
        let output = Command::new(rustc).args(["--version", "--verbose"]).output().ok()?;
        let verbose_version = str::from_utf8(&output.stdout).ok()?;
        Version::parse(verbose_version)
    }

    pub(crate) struct Version {
        pub(crate) nightly: bool,
    }

    impl Version {
        // The known latest stable version. If we unable to determine
        // the rustc version, we assume this is the current version.
        // It is no problem if this is older than the actual latest stable.
        pub(crate) const LATEST: Self = Self::stable();

        const fn stable() -> Self {
            Self { nightly: false }
        }

        pub(crate) fn parse(verbose_version: &str) -> Option<Self> {
            let mut release = verbose_version
                .lines()
                .find(|line| line.starts_with("release: "))
                .map(|line| &line["release: ".len()..])?
                .splitn(2, '-');
            let _version = release.next().unwrap();
            let channel = release.next().unwrap_or_default();
            let nightly = channel == "nightly" || channel == "dev";

            Some(Self { nightly })
        }
    }
}
use version::{rustc_version, Version};
