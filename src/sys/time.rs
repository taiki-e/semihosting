// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::cast_sign_loss)]

// Based on https://github.com/rust-lang/rust/blob/1.92.0/library/std/src/sys/pal/unix/time.rs.

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
                (sub_ge_to_unsigned(self.tv_sec, other.tv_sec), self.tv_nsec.0 - other.tv_nsec.0)
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
        target_arch = "loongarch32",
        target_arch = "loongarch64",
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
