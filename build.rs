// SPDX-License-Identifier: Apache-2.0 OR MIT

// The rustc-cfg emitted by the build script are *not* public API.

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let version = match rustc_version() {
        Some(version) => version,
        None => {
            if env::var_os("SEMIHOSTING_DENY_WARNINGS").unwrap_or_default() == "1" {
                panic!("unable to determine rustc version")
            }
            println!(
                "cargo:warning={}: unable to determine rustc version; assuming latest stable rustc",
                env!("CARGO_PKG_NAME"),
            );
            Version::LATEST
        }
    };

    if version.minor >= 80 {
        // Custom cfgs set by build script. Not public API.
        // grep -E 'cargo:rustc-cfg=' build.rs | grep -v '=//' | sed -E 's/^.*cargo:rustc-cfg=//; s/(=\\)?".*$//' | LC_ALL=C sort -u | tr '\n' ','
        println!(
            "cargo:rustc-check-cfg=cfg(semihosting_no_error_in_core,semihosting_target_feature)"
        );
        // TODO: handle multi-line target_feature_fallback
        // grep -E 'target_feature_fallback\("' build.rs | sed -E 's/^.*target_feature_fallback\(//; s/",.*$/"/' | LC_ALL=C sort -u | tr '\n' ','
        println!(
            r#"cargo:rustc-check-cfg=cfg(semihosting_target_feature,values("mclass","thumb-mode"))"#
        );
    }

    // Note that this is `no_`*, not `has_*`. This allows treating as the latest
    // stable rustc is used when the build script doesn't run. This is useful
    // for non-cargo build systems that don't run the build script.
    // error_in_core stabilized in Rust 1.81 (nightly-2024-06-09): https://github.com/rust-lang/rust/pull/125951
    if !version.probe(81, 2024, 6, 8) {
        println!("cargo:rustc-cfg=semihosting_no_error_in_core");
    }

    let target_arch = &*env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
    if target_arch == "arm" {
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
            // armv7-linux-androideabi and armv7-sony-vita-newlibeabihf are also enable +thumb-mode.
            // https://github.com/rust-lang/rust/blob/1.80.0/compiler/rustc_target/src/spec/targets/armv7_linux_androideabi.rs#L27
            // https://github.com/rust-lang/rust/blob/1.80.0/compiler/rustc_target/src/spec/targets/armv7_sony_vita_newlibeabihf.rs#L39
            let thumb_mode = target.starts_with("thumb")
                || target == "armv7-linux-androideabi"
                || target == "armv7-sony-vita-newlibeabihf";
            target_feature_fallback("thumb-mode", thumb_mode);
        }
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

mod version {
    use std::{env, iter, process::Command, str};

    pub(crate) fn rustc_version() -> Option<Version> {
        let rustc = env::var_os("RUSTC")?;
        let rustc_wrapper = env::var_os("RUSTC_WRAPPER").filter(|v| !v.is_empty());
        // Do not apply RUSTC_WORKSPACE_WRAPPER: https://github.com/cuviper/autocfg/issues/58#issuecomment-2067625980
        let mut rustc = rustc_wrapper.into_iter().chain(iter::once(rustc));
        let mut cmd = Command::new(rustc.next().unwrap());
        cmd.args(rustc);
        // Use verbose version output because the packagers add extra strings to the normal version output.
        // Do not use long flags (--version --verbose) because clippy-deriver doesn't handle them properly.
        // -vV is also matched with that cargo internally uses: https://github.com/rust-lang/cargo/blob/14b46ecc62aa671d7477beba237ad9c6a209cf5d/src/cargo/util/rustc.rs#L65
        let output = cmd.arg("-vV").output().ok()?;
        let verbose_version = str::from_utf8(&output.stdout).ok()?;
        Version::parse(verbose_version)
    }

    pub(crate) struct Version {
        pub(crate) minor: u32,
        pub(crate) nightly: bool,
        commit_date: Date,
    }

    impl Version {
        // The known latest stable version. If we unable to determine
        // the rustc version, we assume this is the current version.
        // It is no problem if this is older than the actual latest stable.
        pub(crate) const LATEST: Self = Self::stable(80);

        const fn stable(minor: u32) -> Self {
            Self { minor, nightly: false, commit_date: Date::UNKNOWN }
        }

        pub(crate) fn probe(&self, minor: u32, year: u16, month: u8, day: u8) -> bool {
            if self.nightly {
                self.minor > minor
                    || self.minor == minor && self.commit_date >= Date::new(year, month, day)
            } else {
                self.minor >= minor
            }
        }

        pub(crate) fn parse(verbose_version: &str) -> Option<Self> {
            let mut release = verbose_version
                .lines()
                .find(|line| line.starts_with("release: "))
                .map(|line| &line["release: ".len()..])?
                .splitn(2, '-');
            let version = release.next().unwrap();
            let channel = release.next().unwrap_or_default();
            let mut digits = version.splitn(3, '.');
            let major = digits.next()?;
            if major != "1" {
                return None;
            }
            let minor = digits.next()?.parse::<u32>().ok()?;
            let _patch = digits.next().unwrap_or("0").parse::<u32>().ok()?;
            let nightly = channel == "nightly" || channel == "dev";

            // we don't refer commit date on stable/beta.
            if nightly {
                let commit_date = (|| {
                    let mut commit_date = verbose_version
                        .lines()
                        .find(|line| line.starts_with("commit-date: "))
                        .map(|line| &line["commit-date: ".len()..])?
                        .splitn(3, '-');
                    let year = commit_date.next()?.parse::<u16>().ok()?;
                    let month = commit_date.next()?.parse::<u8>().ok()?;
                    let day = commit_date.next()?.parse::<u8>().ok()?;
                    if month > 12 || day > 31 {
                        return None;
                    }
                    Some(Date::new(year, month, day))
                })();
                Some(Self { minor, nightly, commit_date: commit_date.unwrap_or(Date::UNKNOWN) })
            } else {
                Some(Self::stable(minor))
            }
        }
    }

    #[derive(PartialEq, PartialOrd)]
    struct Date {
        year: u16,
        month: u8,
        day: u8,
    }

    impl Date {
        const UNKNOWN: Self = Self::new(0, 0, 0);

        const fn new(year: u16, month: u8, day: u8) -> Self {
            Self { year, month, day }
        }
    }
}
use version::{rustc_version, Version};
