// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target = &*env::var("TARGET").expect("TARGET not set");
    let target_arch = &*env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
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
                "cargo:warning={}: unable to determine rustc version; assuming latest stable rustc (1.{})",
                env!("CARGO_PKG_NAME"),
                Version::LATEST.minor
            );
            Version::LATEST
        }
    };

    if version.nightly && is_allowed_feature("rustc_attrs") {
        println!("cargo:rustc-cfg=semihosting_unstable_rustc_attrs");
    }

    if target_arch == "arm" {
        if target.starts_with("thumb") {
            target_feature_if("thumb-mode", true, &version, None, true)
        }
        // See portable-atomic's build.rs for more
        let mut subarch =
            target.strip_prefix("arm").or_else(|| target.strip_prefix("thumb")).unwrap();
        subarch = subarch.strip_prefix("eb").unwrap_or(subarch); // ignore endianness
        subarch = subarch.split('-').next().unwrap(); // ignore vender/os/env
        subarch = subarch.split('.').next().unwrap(); // ignore .base/.main suffix
        let mut known = true;
        // See https://github.com/taiki-e/atomic-maybe-uninit/blob/HEAD/build.rs for details
        let mut is_mclass = false;
        match subarch {
            "v7" | "v7a" | "v7neon" | "v7s" | "v7k" | "v8a" => {} // aclass
            "v6m" | "v7em" | "v7m" | "v8m" => is_mclass = true,
            "v7r" | "v8r" => {} // rclass
            // arm-linux-androideabi is v5te
            // https://github.com/rust-lang/rust/blob/1.68.0/compiler/rustc_target/src/spec/arm_linux_androideabi.rs#L11-L12
            _ if target == "arm-linux-androideabi" => subarch = "v5te",
            // v6 targets other than v6m don't have *class target feature.
            "" | "v6" | "v6k" => subarch = "v6",
            // Other targets don't have *class target feature.
            "v4t" | "v5te" => {}
            _ => {
                known = false;
                println!(
                    "cargo:warning={}: unrecognized arm subarch: {}",
                    env!("CARGO_PKG_NAME"),
                    target
                );
            }
        }
        let (v8, v8m) = if known && subarch.starts_with("v8") {
            // ARMv8-M Mainline/Baseline are not considered as v8 by rustc.
            // https://github.com/rust-lang/stdarch/blob/a0c30f3e3c75adcd6ee7efc94014ebcead61c507/crates/core_arch/src/arm_shared/mod.rs
            if subarch.starts_with("v8m") {
                (false, true)
            } else {
                (true, false)
            }
        } else {
            (false, false)
        };
        target_feature_if("mclass", is_mclass, &version, None, true);
        target_feature_if("v8", v8, &version, None, true);
        target_feature_if("v8m", v8m, &version, None, false);
    }
}

fn target_feature_if(
    name: &str,
    mut has_target_feature: bool,
    version: &Version,
    stabilized: Option<u32>,
    is_rustc_target_feature: bool,
) {
    // HACK: Currently, it seems that the only way to handle unstable target
    // features on the stable is to parse the `-C target-feature` in RUSTFLAGS.
    //
    // - #[cfg(target_feature = "unstable_target_feature")] doesn't work on stable.
    // - CARGO_CFG_TARGET_FEATURE excludes unstable target features on stable.
    //
    // As mentioned in the [RFC2045], unstable target features are also passed to LLVM
    // (e.g., https://godbolt.org/z/8Eh3z5Wzb), so this hack works properly on stable.
    //
    // [RFC2045]: https://rust-lang.github.io/rfcs/2045-target-feature.html#backend-compilation-options
    if is_rustc_target_feature
        && (version.nightly || stabilized.map_or(false, |stabilized| version.minor >= stabilized))
    {
        // In this case, cfg(target_feature = "...") would work, so skip emitting our own target_feature cfg.
        return;
    } else if let Some(rustflags) = env::var_os("CARGO_ENCODED_RUSTFLAGS") {
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
        println!("cargo:rustc-cfg=semihosting_target_feature=\"{}\"", name);
    }
}

fn is_allowed_feature(name: &str) -> bool {
    // allowed by default
    let mut allowed = true;
    if let Some(rustflags) = env::var_os("CARGO_ENCODED_RUSTFLAGS") {
        for mut flag in rustflags.to_string_lossy().split('\x1f') {
            flag = flag.strip_prefix("-Z").unwrap_or(flag);
            if let Some(flag) = flag.strip_prefix("allow-features=") {
                // If it is specified multiple times, the last value will be preferred.
                allowed = flag.split(',').any(|allowed| allowed == name);
            }
        }
    }
    allowed
}

mod version {
    use std::{env, process::Command, str};

    pub(crate) fn rustc_version() -> Option<Version> {
        let rustc = env::var_os("RUSTC")?;
        // Use verbose version output because the packagers add extra strings to the normal version output.
        let output = Command::new(rustc).args(["--version", "--verbose"]).output().ok()?;
        let output = str::from_utf8(&output.stdout).ok()?;
        Version::parse(output)
    }

    #[cfg_attr(test, derive(Debug, PartialEq))]
    pub(crate) struct Version {
        pub(crate) minor: u32,
        pub(crate) nightly: bool,
        // commit_date: Date,
    }

    impl Version {
        // The known latest stable version. If we unable to determine
        // the rustc version, we assume this is the current version.
        // It is no problem if this is older than the actual latest stable.
        pub(crate) const LATEST: Self = Self::stable(68);

        pub(crate) const fn stable(rustc_minor: u32) -> Self {
            Self {
                minor: rustc_minor,
                nightly: false,
                // commit_date: Date::UNKNOWN,
            }
        }

        // pub(crate) fn probe(&self, minor: u32, year: u16, month: u8, day: u8) -> bool {
        //     if self.nightly {
        //         self.minor > minor || self.commit_date >= Date::new(year, month, day)
        //     } else {
        //         self.minor >= minor
        //     }
        // }

        pub(crate) fn parse(text: &str) -> Option<Self> {
            let mut release = text
                .lines()
                .find(|line| line.starts_with("release: "))
                .map(|line| &line["release: ".len()..])?
                .splitn(2, '-');
            let version = release.next().unwrap();
            let channel = release.next().unwrap_or_default();
            let mut digits = version.splitn(3, '.');
            let major = digits.next()?.parse::<u32>().ok()?;
            if major != 1 {
                return None;
            }
            let minor = digits.next()?.parse::<u32>().ok()?;
            let _patch = digits.next().unwrap_or("0").parse::<u32>().ok()?;
            let nightly = channel == "nightly" || channel == "dev";

            // we don't refer commit date on stable/beta.
            if nightly {
                // let commit_date = (|| {
                //     let mut commit_date = text
                //         .lines()
                //         .find(|line| line.starts_with("commit-date: "))
                //         .map(|line| &line["commit-date: ".len()..])?
                //         .splitn(3, '-');
                //     let year = commit_date.next()?.parse::<u16>().ok()?;
                //     let month = commit_date.next()?.parse::<u8>().ok()?;
                //     let day = commit_date.next()?.parse::<u8>().ok()?;
                //     if month > 12 || day > 31 {
                //         return None;
                //     }
                //     Some(Date::new(year, month, day))
                // })();
                Some(Version {
                    minor,
                    nightly,
                    // commit_date: commit_date.unwrap_or(Date::UNKNOWN),
                })
            } else {
                Some(Version::stable(minor))
            }
        }
    }

    // #[derive(PartialEq, PartialOrd)]
    // pub(crate) struct Date {
    //     pub(crate) year: u16,
    //     pub(crate) month: u8,
    //     pub(crate) day: u8,
    // }

    // impl Date {
    //     const UNKNOWN: Self = Self::new(0, 0, 0);

    //     const fn new(year: u16, month: u8, day: u8) -> Self {
    //         Self { year, month, day }
    //     }
    // }
}
use version::{rustc_version, Version};
