// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Experimental APIs.
//!
//! Note: All APIs in this module are experimental and outside of the normal semver guarantees and
//! minor or patch versions of semihosting may make breaking changes to them at any time.

#[cfg(feature = "args")]
#[cfg_attr(semihosting_doc_cfg, doc(cfg(feature = "args")))]
pub mod env;
#[cfg(feature = "panic-unwind")]
#[cfg_attr(semihosting_doc_cfg, doc(cfg(feature = "panic-unwind")))]
pub mod panic;
#[cfg(feature = "time")]
#[cfg_attr(semihosting_doc_cfg, doc(cfg(feature = "time")))]
pub mod time;
