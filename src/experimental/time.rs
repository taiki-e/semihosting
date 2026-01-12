// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Temporal quantification.
//!
//! This module is a subset of the [`std::time`] module.
//!
//! [`std::time`]: https://doc.rust-lang.org/std/time/index.html

// Based on https://github.com/rust-lang/rust/blob/1.92.0/library/std/src/time.rs.

#[doc(no_inline)]
pub use core::time::Duration;
#[cfg(not(semihosting_no_duration_checked_float))]
#[doc(no_inline)]
pub use core::time::TryFromFloatSecsError;
use core::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::sys::time;

/// A measurement of a monotonically nondecreasing clock.
/// Opaque and useful only with [`Duration`].
///
/// See [`std::time::Instant` documentation][std] for details.
///
/// [std]: https://doc.rust-lang.org/std/time/struct.Instant.html
///
/// # Platform-specific behavior
///
/// An `Instant` is a wrapper around system-specific types and it may behave
/// differently depending on the underlying system.
///
/// The following semihosting calls are currently being used by `now()` to find out
/// the current time:
///
/// | Platform                                                      | Semihosting call | Representable precision  |
/// | ------------------------------------------------------------- | ---------------- | ------------------------ |
/// | AArch64, Arm, RISC-V, LoongArch, Xtensa (openocd-semihosting) | [SYS_CLOCK]      | 10 millisecond intervals |
/// | MIPS32, MIPS64                                                | (Unsupported)    | -                        |
///
/// [SYS_CLOCK]: https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-clock-0x10
///
/// **Disclaimer:** These semihosting calls might change over time.
///
/// > Note: mathematical operations like [`add`] may panic if the underlying
/// > structure cannot represent the new point in time.
///
/// [`add`]: Instant::add
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(time::Instant);

/// A measurement of the system clock, useful for talking to
/// external entities like the file system or other processes.
///
/// See [`std::time::SystemTime` documentation][std] for details.
///
/// [std]: https://doc.rust-lang.org/std/time/struct.SystemTime.html
///
/// # Platform-specific behavior
///
/// The precision of `SystemTime` can depend on the underlying system.
///
/// The following semihosting calls are currently being used by `now()` to find out
/// the current time:
///
/// | Platform                                                      | Semihosting call | Representable precision  |
/// | ------------------------------------------------------------- | ---------------- | ------------------------ |
/// | AArch64, Arm, RISC-V, LoongArch, Xtensa (openocd-semihosting) | [SYS_TIME]       | second intervals         |
/// | MIPS32, MIPS64                                                | (Unsupported)    | -                        |
///
/// [SYS_TIME]: https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-time-0x11
///
/// **Disclaimer:** These semihosting calls might change over time.
///
/// > Note: mathematical operations like [`add`] may panic if the underlying
/// > structure cannot represent the new point in time.
///
/// [`add`]: SystemTime::add
/// [`UNIX_EPOCH`]: SystemTime::UNIX_EPOCH
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime(time::SystemTime);

/// An error returned from the `duration_since` and `elapsed` methods on
/// `SystemTime`, used to learn how far in the opposite direction a system time
///
/// See [`std::time::SystemTimeError` documentation][std] for details.
///
/// [std]: https://doc.rust-lang.org/std/time/struct.SystemTimeError.html
#[derive(Clone, Debug)]
pub struct SystemTimeError(Duration);

impl Instant {
    // TODO(time): return result?
    /// Returns an instant corresponding to "now".
    #[must_use]
    pub fn now() -> Self {
        Self(time::Instant::now().unwrap())
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is later than this one.
    ///
    /// See [`std::time::Instant::duration_since` documentation][std] for details.
    ///
    /// [std]: https://doc.rust-lang.org/std/time/struct.Instant.html#method.duration_since
    #[must_use]
    pub fn duration_since(&self, earlier: Self) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or None if that instant is later than this one.
    ///
    /// See [`std::time::Instant::checked_duration_since` documentation][std] for details.
    ///
    /// [std]: https://doc.rust-lang.org/std/time/struct.Instant.html#method.checked_duration_since
    #[must_use]
    pub fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
        self.0.checked_sub_instant(&earlier.0)
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is later than this one.
    ///
    /// See [`std::time::Instant::saturating_duration_since` documentation][std] for details.
    ///
    /// [std]: https://doc.rust-lang.org/std/time/struct.Instant.html#method.saturating_duration_since
    #[must_use]
    pub fn saturating_duration_since(&self, earlier: Self) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }

    /// Returns the amount of time elapsed since this instant.
    ///
    /// See [`std::time::Instant::elapsed` documentation][std] for details.
    ///
    /// [std]: https://doc.rust-lang.org/std/time/struct.Instant.html#method.elapsed
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        Self::now() - *self
    }

    /// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be represented as
    /// `Instant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        self.0.checked_add_duration(&duration).map(Self)
    }

    /// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be represented as
    /// `Instant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        self.0.checked_sub_duration(&duration).map(Self)
    }
}

impl Add<Duration> for Instant {
    type Output = Self;

    /// # Panics
    ///
    /// This function may panic if the resulting point in time cannot be represented by the
    /// underlying data structure. See [`Instant::checked_add`] for a version without panic.
    fn add(self, other: Duration) -> Self {
        self.checked_add(other).expect("overflow when adding duration to instant")
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl Sub<Duration> for Instant {
    type Output = Self;

    fn sub(self, other: Duration) -> Self::Output {
        self.checked_sub(other).expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is later than this one.
    ///
    /// See [`std::time::Instant` documentation][std] for details.
    ///
    /// [std]: https://doc.rust-lang.org/std/time/struct.Instant.html#impl-Sub-for-Instant
    fn sub(self, other: Self) -> Self::Output {
        self.duration_since(other)
    }
}

impl fmt::Debug for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl SystemTime {
    /// An anchor in time which can be used to create new `SystemTime` instances or
    /// learn about where in time a `SystemTime` lies.
    ///
    /// See [`std::time::SystemTime::UNIX_EPOCH` documentation][std] for details.
    ///
    /// [std]: https://doc.rust-lang.org/std/time/struct.SystemTime.html#associatedconstant.UNIX_EPOCH
    pub const UNIX_EPOCH: Self = Self(time::UNIX_EPOCH);

    // TODO(time): return result?
    /// Returns the system time corresponding to "now".
    #[must_use]
    pub fn now() -> Self {
        Self(time::SystemTime::now().unwrap())
    }

    /// Returns the amount of time elapsed from an earlier point in time.
    ///
    /// See [`std::time::SystemTime::duration_since` documentation][std] for details.
    ///
    /// [std]: https://doc.rust-lang.org/std/time/struct.SystemTime.html#method.duration_since
    pub fn duration_since(&self, earlier: Self) -> Result<Duration, SystemTimeError> {
        self.0.sub_time(&earlier.0).map_err(SystemTimeError)
    }

    /// Returns the difference from this system time to the
    /// current clock time.
    ///
    /// See [`std::time::SystemTime::elapsed` documentation][std] for details.
    ///
    /// [std]: https://doc.rust-lang.org/std/time/struct.SystemTime.html#method.elapsed
    pub fn elapsed(&self) -> Result<Duration, SystemTimeError> {
        Self::now().duration_since(*self)
    }

    /// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be represented as
    /// `SystemTime` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        self.0.checked_add_duration(&duration).map(Self)
    }

    /// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be represented as
    /// `SystemTime` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        self.0.checked_sub_duration(&duration).map(Self)
    }
}

impl Add<Duration> for SystemTime {
    type Output = Self;

    /// # Panics
    ///
    /// This function may panic if the resulting point in time cannot be represented by the
    /// underlying data structure. See [`SystemTime::checked_add`] for a version without panic.
    fn add(self, dur: Duration) -> Self::Output {
        self.checked_add(dur).expect("overflow when adding duration to instant")
    }
}

impl AddAssign<Duration> for SystemTime {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl Sub<Duration> for SystemTime {
    type Output = Self;

    fn sub(self, dur: Duration) -> Self::Output {
        self.checked_sub(dur).expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<Duration> for SystemTime {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}

impl fmt::Debug for SystemTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl SystemTimeError {
    /// Returns the positive duration which represents how far forward the
    /// second system time was from the first.
    ///
    /// See [`std::time::SystemTimeError::duration` documentation][std] for details.
    ///
    /// [std]: https://doc.rust-lang.org/std/time/struct.SystemTimeError.html#method.duration
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.0
    }
}

#[cfg(not(semihosting_no_error_in_core))]
impl core::error::Error for SystemTimeError {}

impl fmt::Display for SystemTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("second time provided was later than self")
    }
}
