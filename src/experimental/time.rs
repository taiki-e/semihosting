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

use self::sys as time;

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
/// The following system calls are currently being used by `now()` to find out
/// the current time:
///
/// | Platform                  | System call   | Representable precision  |
/// | ------------------------- | ------------- | ------------------------ |
/// | AArch64/Arm/RISC-V/Xtensa | [SYS_CLOCK]   | 10 millisecond intervals |
/// | MIPS32/MIPS64             | (Unsupported) | -                        |
///
/// [SYS_CLOCK]: https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-clock-0x10
///
/// **Disclaimer:** These system calls might change over time.
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
/// The following system calls are currently being used by `now()` to find out
/// the current time:
///
/// | Platform                  | System call   | Representable precision  |
/// | ------------------------- | ------------- | ------------------------ |
/// | AArch64/Arm/RISC-V/Xtensa | [SYS_TIME]    | second intervals         |
/// | MIPS32/MIPS64             | (Unsupported) | -                        |
///
/// [SYS_TIME]: https://github.com/ARM-software/abi-aa/blob/2025Q1/semihosting/semihosting.rst#sys-time-0x11
///
/// **Disclaimer:** These system calls might change over time.
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
    // TODO: return result?
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

    // TODO: return result?
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

// TODO: move to sys/time.rs
// Based on https://github.com/rust-lang/rust/blob/1.92.0/library/std/src/sys/pal/unix/time.rs.
mod sys {
    #![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::cast_sign_loss)]

    use core::{fmt, time::Duration};

    use crate::io;

    const NSEC_PER_SEC: u64 = 1_000_000_000;
    pub(crate) const UNIX_EPOCH: SystemTime = SystemTime { t: Timespec::zero() };

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    struct Nanoseconds(u32);

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub(crate) struct SystemTime {
        t: Timespec,
    }

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct Timespec {
        tv_sec: i64,
        tv_nsec: Nanoseconds,
    }

    impl SystemTime {
        pub(crate) fn sub_time(&self, other: &Self) -> Result<Duration, Duration> {
            self.t.sub_timespec(&other.t)
        }

        pub(crate) fn checked_add_duration(&self, other: &Duration) -> Option<Self> {
            Some(Self { t: self.t.checked_add_duration(other)? })
        }

        pub(crate) fn checked_sub_duration(&self, other: &Duration) -> Option<Self> {
            Some(Self { t: self.t.checked_sub_duration(other)? })
        }
    }

    impl fmt::Debug for SystemTime {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("SystemTime")
                .field("tv_sec", &self.t.tv_sec)
                .field("tv_nsec", &self.t.tv_nsec.0)
                .finish()
        }
    }

    impl Timespec {
        const fn new_unchecked(tv_sec: i64, tv_nsec: i64) -> Self {
            Self { tv_sec, tv_nsec: Nanoseconds(tv_nsec as u32) }
        }

        const fn zero() -> Timespec {
            Self::new_unchecked(0, 0)
        }

        fn sub_timespec(&self, other: &Timespec) -> Result<Duration, Duration> {
            // When a >= b, the difference fits in u64.
            fn sub_ge_to_unsigned(a: i64, b: i64) -> u64 {
                debug_assert!(a >= b);
                a.wrapping_sub(b) as u64
            }

            if self >= other {
                let (secs, nsec) = if self.tv_nsec.0 >= other.tv_nsec.0 {
                    (
                        sub_ge_to_unsigned(self.tv_sec, other.tv_sec),
                        self.tv_nsec.0 - other.tv_nsec.0,
                    )
                } else {
                    // Following sequence of assertions explain why `self.tv_sec - 1` does not underflow.
                    debug_assert!(self.tv_nsec < other.tv_nsec);
                    debug_assert!(self.tv_sec > other.tv_sec);
                    debug_assert!(self.tv_sec > i64::MIN);
                    (
                        sub_ge_to_unsigned(self.tv_sec - 1, other.tv_sec),
                        self.tv_nsec.0 + (NSEC_PER_SEC as u32) - other.tv_nsec.0,
                    )
                };

                Ok(Duration::new(secs, nsec))
            } else {
                match other.sub_timespec(self) {
                    Ok(d) => Err(d),
                    Err(d) => Ok(d),
                }
            }
        }

        pub(crate) fn checked_add_duration(&self, other: &Duration) -> Option<Timespec> {
            // i*::checked_add_unsigned requires Rust 1.66
            #[inline]
            fn checked_add_unsigned(this: i64, rhs: u64) -> Option<i64> {
                let (a, b) = {
                    let rhs = rhs as i64;
                    let (res, overflowed) = this.overflowing_add(rhs);
                    (res, overflowed ^ (rhs < 0))
                };
                if b { None } else { Some(a) }
            }

            let mut secs = checked_add_unsigned(self.tv_sec, other.as_secs())?;

            // Nano calculations can't overflow because nanos are <1B which fit
            // in a u32.
            let mut nsec = other.subsec_nanos() + self.tv_nsec.0;
            if nsec >= NSEC_PER_SEC as u32 {
                nsec -= NSEC_PER_SEC as u32;
                secs = secs.checked_add(1)?;
            }
            Some(Timespec::new_unchecked(secs, nsec.into()))
        }

        pub(crate) fn checked_sub_duration(&self, other: &Duration) -> Option<Timespec> {
            // i*::checked_sub_unsigned requires Rust 1.66
            #[inline]
            fn checked_sub_unsigned(this: i64, rhs: u64) -> Option<i64> {
                let (a, b) = {
                    let rhs = rhs as i64;
                    let (res, overflowed) = this.overflowing_sub(rhs);
                    (res, overflowed ^ (rhs < 0))
                };
                if b { None } else { Some(a) }
            }

            let mut secs = checked_sub_unsigned(self.tv_sec, other.as_secs())?;

            // Similar to above, nanos can't overflow.
            let mut nsec = self.tv_nsec.0 as i32 - other.subsec_nanos() as i32;
            if nsec < 0 {
                nsec += NSEC_PER_SEC as i32;
                secs = secs.checked_sub(1)?;
            }
            Some(Timespec::new_unchecked(secs, nsec.into()))
        }
    }

    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub(crate) struct Instant {
        t: Timespec,
    }

    impl Instant {
        pub(crate) fn checked_sub_instant(&self, other: &Instant) -> Option<Duration> {
            self.t.sub_timespec(&other.t).ok()
        }

        pub(crate) fn checked_add_duration(&self, other: &Duration) -> Option<Instant> {
            Some(Instant { t: self.t.checked_add_duration(other)? })
        }

        pub(crate) fn checked_sub_duration(&self, other: &Duration) -> Option<Instant> {
            Some(Instant { t: self.t.checked_sub_duration(other)? })
        }
    }

    impl fmt::Debug for Instant {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Instant")
                .field("tv_sec", &self.t.tv_sec)
                .field("tv_nsec", &self.t.tv_nsec.0)
                .finish()
        }
    }

    cfg_sel!({
        #[cfg(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv32",
            target_arch = "riscv64",
            all(target_arch = "xtensa", feature = "openocd-semihosting"),
        ))]
        {
            use crate::sys::arm_compat::{sys_clock, sys_time};

            const CENTISECONDS_PER_SEC: u64 = 100;
            const NANOS_PER_CENTI: u32 = 10_000_000;

            impl SystemTime {
                pub(crate) fn now() -> io::Result<Self> {
                    // SYS_TIME doesn't have Y2038 problem (although it still has Y2106 problem): https://github.com/ARM-software/abi-aa/commit/d281283bf3dcec4d4ebf9e5646020d77904904e1
                    let tv_sec = sys_time()? as u64 as i64;
                    // SYS_TIME returns seconds, so tv_nsec is always zero.
                    let tv_nsec = Nanoseconds(0);
                    Ok(Self { t: Timespec { tv_sec, tv_nsec } })
                }
            }
            impl Instant {
                pub(crate) fn now() -> io::Result<Self> {
                    // SYS_CLOCK returns centiseconds (hundredths of a second).
                    // Conversion is based on Duration::from_millis.
                    let centiseconds = sys_clock()? as u64;
                    let secs = centiseconds / CENTISECONDS_PER_SEC;
                    let subsec_centiseconds = (centiseconds % CENTISECONDS_PER_SEC) as u32;
                    // SAFETY: (x % 10_000_000) * 100 < 1_000_000_000
                    //         => x % 10_000_000 < 10_000_000
                    let subsec_nanos = Nanoseconds(subsec_centiseconds * NANOS_PER_CENTI);
                    Ok(Self { t: Timespec { tv_sec: secs as i64, tv_nsec: subsec_nanos } })
                }
            }
        }
        #[cfg(else)]
        {
            impl SystemTime {
                pub(crate) fn now() -> io::Result<Self> {
                    Err(io::ErrorKind::Unsupported.into())
                }
            }
            impl Instant {
                pub(crate) fn now() -> io::Result<Self> {
                    Err(io::ErrorKind::Unsupported.into())
                }
            }
        }
    });
}
