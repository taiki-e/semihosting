// SPDX-License-Identifier: Apache-2.0 OR MIT

// rustflags parsing used in the build script.
//
// Adapted from https://github.com/taiki-e/atomic-maybe-uninit

use std::{vec, vec::Vec};

// Note that this doesn't catch the target-feature/target-cpu set via target spec,
// and -Z allow-features passed via CLI (https://github.com/rust-lang/cargo/issues/17065).
pub(crate) struct Rustflags<'a> {
    // -C target-feature=...
    pub(crate) target_feature: Vec<(bool, &'a [u8])>,
}

impl<'a> Rustflags<'a> {
    pub(crate) fn new(rustflags: &'a str) -> Self {
        let mut target_feature = vec![];
        let mut rustflags = rustflags.split('\x1f');
        // https://github.com/rust-lang/rust/blob/1.100.0/compiler/rustc_session/src/config.rs#L2001
        while let Some(mut flag) = rustflags.next() {
            // rustc supports argfile via @<path> syntax, and it is accepted within rustflags, but
            // Cargo does not support it very well, and the fact that it is accepted might not be
            // intentional, so ignore it here for now.
            // https://github.com/rust-lang/cargo/issues/17369
            if flag.as_bytes().first() != Some(&b'-') {
                continue;
            }
            let mut start = 1;
            let mut long = false;
            if flag.as_bytes().get(start) == Some(&b'-') {
                start += 1;
                long = true;
            } else {
                // Handle known short flags without value.
                while matches!(flag.as_bytes().get(start), Some(b'g' | b'O' | b'v')) {
                    start += 1;
                }
            }
            let b = match flag.as_bytes().get(start) {
                Some(&b) => b,
                None => continue,
            };
            match b {
                // -C / -Z
                b'C' | b'Z' => {
                    start += 1;
                    if flag.len() == start {
                        match rustflags.next() {
                            Some(f) => flag = f,
                            None => break,
                        }
                    } else if long {
                        if flag.as_bytes().get(start) != Some(&b'=') {
                            continue;
                        }
                        flag = &flag[start + 1..];
                    } else {
                        flag = &flag[start..];
                    }
                    if b == b'Z' {
                        continue;
                    }
                }
                // Handle other known short flags with value.
                b'L' | b'l' | b'o' | b'A' | b'W' | b'D' | b'F' | b'j'
                    if flag.len() == start + 1 =>
                {
                    rustflags.next();
                    continue;
                }
                _ => {
                    if !b.is_ascii() || !long {
                        continue;
                    }
                    flag = &flag[start..];
                    // --codegen (long version of -C)
                    if let Some(f) = flag.strip_prefix("codegen") {
                        if let Some(f) = f.strip_prefix('=') {
                            flag = f;
                        } else if f.is_empty() {
                            match rustflags.next() {
                                Some(f) => flag = f,
                                None => break,
                            }
                        } else {
                            continue;
                        }
                    } else if flag.contains('=') {
                        continue;
                    } else {
                        // Handle other known long flags with value.
                        match flag {
                            "cfg" | "check-cfg" | "crate-type" | "crate-name" | "edition"
                            | "emit" | "print" | "out-dir" | "explain" | "target" | "allow"
                            | "warn" | "force-warn" | "deny" | "forbid" | "cap-lints"
                            | "extern" | "sysroot" | "error-format" | "json" | "color"
                            | "diagnostic-width" | "remap-path-prefix" | "remap-path-scope"
                            | "env-set" | "jobs" | "jobs-frontend" | "jobs-backend"
                            | "jobs-linker" => {
                                rustflags.next();
                            }
                            _ => {}
                        }
                        continue;
                    }
                }
            }
            if let Some(flag) = strip_flag(flag, "target-feature=", "target_feature=") {
                for s in flag.split(',') {
                    match s.as_bytes().split_first() {
                        Some((b'+', f)) => target_feature.push((true, f)),
                        Some((b'-', f)) => target_feature.push((false, f)),
                        _ => {}
                    }
                }
            }
        }
        Self { target_feature }
    }
}

fn strip_flag<'a>(
    flag: &'a str,
    prefix: &'static str,
    prefix_underscored: &'static str,
) -> Option<&'a str> {
    if let Some(flag) = flag.strip_prefix(prefix) {
        return Some(flag);
    }
    // rustc accepts both -C target-feature and -C target_feature: https://godbolt.org/z/snfGeKnrv
    flag.strip_prefix(prefix_underscored)
}
