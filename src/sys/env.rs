// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::{cell::Cell, mem::MaybeUninit};

use crate::{io, utils::slice_assume_init_ref};

const NUL: u8 = b'\0';

pub(crate) struct ArgsBytes<const BUF_SIZE: usize> {
    // Invariant: self.buf[..self.size] is initialized.
    buf: [MaybeUninit<u8>; BUF_SIZE],
    next: Cell<usize>,
    size: usize,
    #[cfg(any(
        target_arch = "mips",
        target_arch = "mips32r6",
        target_arch = "mips64",
        target_arch = "mips64r6",
    ))]
    next_fn: for<'a> fn(&'a [u8], &Cell<usize>) -> Option<&'a [u8]>,
}

impl<const BUF_SIZE: usize> ArgsBytes<BUF_SIZE> {
    const UNINIT_BUF: [MaybeUninit<u8>; BUF_SIZE] = [MaybeUninit::uninit(); BUF_SIZE];
    fn init(&self) -> &[u8] {
        // SAFETY: safe due to buf's invariant.
        unsafe { slice_assume_init_ref(self.buf.get_unchecked(..self.size)) }
    }
}

/// Arguments read into a buffer owned by the caller.
///
/// Unlike [`ArgsBytes`], this never copies the buffer, so the peak stack use is
/// the buffer itself rather than twice it.
pub(crate) struct ArgsBytesRef<'a> {
    buf: &'a [u8],
    next: Cell<usize>,
    #[cfg(any(
        target_arch = "mips",
        target_arch = "mips32r6",
        target_arch = "mips64",
        target_arch = "mips64r6",
    ))]
    next_fn: for<'b> fn(&'b [u8], &Cell<usize>) -> Option<&'b [u8]>,
}

pub(crate) fn next_from_cmdline<'a>(buf: &'a [u8], next: &Cell<usize>) -> Option<&'a [u8]> {
    // Implementations disagree on whether the length reported by SYS_GET_CMDLINE
    // covers the trailing nul. The command line is nul-terminated either way, so
    // strip the terminator when covered.
    let buf = match buf.split_last() {
        Some((&NUL, buf)) => buf,
        _ => buf,
    };
    if next.get() >= buf.len() {
        return None;
    }
    let mut start = next.get();
    let mut end = None;
    let is_blank = |b: u8| b == b' ' || b == b'\t';
    let mut delim = NUL;
    let mut in_argument = false;
    while next.get() < buf.len() {
        let b = buf[next.get()];
        if !in_argument {
            if is_blank(b) {
                end = Some(next.get());
                next.set(next.get() + 1);
                break;
            }
            if b == b'"' || b == b'\'' {
                delim = b;
                start += 1;
            }
            in_argument = true;
        } else if delim != NUL {
            if b == delim {
                end = Some(next.get());
                next.set(next.get() + 2);
                break;
            }
        } else if is_blank(b) {
            end = Some(next.get());
            next.set(next.get() + 1);
            break;
        }

        next.set(next.get() + 1);
    }
    Some(&buf[start..end.unwrap_or_else(|| next.get())])
}

#[cfg(any(
    test,
    target_arch = "mips",
    target_arch = "mips32r6",
    target_arch = "mips64",
    target_arch = "mips64r6",
))]
#[cfg_attr(test, allow(dead_code))] // TODO(env): unit test
fn next_from_args<'a>(buf: &'a [u8], next: &Cell<usize>) -> Option<&'a [u8]> {
    if next.get() >= buf.len() {
        return None;
    }
    let start = next.get();
    let mut end = None;
    while next.get() < buf.len() {
        let b = buf[next.get()];
        if b == NUL {
            end = Some(next.get());
            next.set(next.get() + 1);
            break;
        }
        next.set(next.get() + 1);
    }
    let end = end.unwrap_or_else(|| next.get());
    let last = end.saturating_sub(1);
    if start != last {
        let start_b = buf[start];
        let last_b = buf[last];
        if start_b == b'"' && last_b == b'"' || start_b == b'\'' && last_b == b'\'' {
            return Some(&buf[start + 1..last]);
        }
    }
    Some(&buf[start..end])
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
        use crate::sys::arm_compat::sys_get_cmdline_uninit;

        pub(crate) fn args_bytes<const BUF_SIZE: usize>() -> io::Result<ArgsBytes<BUF_SIZE>> {
            let mut buf = ArgsBytes::<BUF_SIZE>::UNINIT_BUF;
            let size = sys_get_cmdline_uninit(&mut buf)?.len();
            Ok(ArgsBytes { buf, next: Cell::new(0), size })
        }
        pub(crate) fn args_bytes_in(buf: &mut [MaybeUninit<u8>]) -> io::Result<ArgsBytesRef<'_>> {
            let buf = sys_get_cmdline_uninit(buf)?;
            Ok(ArgsBytesRef { buf, next: Cell::new(0) })
        }
        pub(crate) fn next<const BUF_SIZE: usize>(args: &ArgsBytes<BUF_SIZE>) -> Option<&[u8]> {
            next_from_cmdline(args.init(), &args.next)
        }
        pub(crate) fn next_ref<'a>(args: &ArgsBytesRef<'a>) -> Option<&'a [u8]> {
            next_from_cmdline(args.buf, &args.next)
        }
    }
    #[cfg(any(
        target_arch = "mips",
        target_arch = "mips32r6",
        target_arch = "mips64",
        target_arch = "mips64r6",
    ))]
    {
        use crate::sys::mips::{mips_argc, mips_argn, mips_argnlen};

        /// Reads every argument into `buf`, returning the argument count and the
        /// number of bytes written.
        fn read_args(buf: &mut [MaybeUninit<u8>]) -> io::Result<(usize, usize)> {
            let argc = mips_argc();
            let mut start: usize = 0;
            for i in 0..argc {
                let len = mips_argnlen(i)?.saturating_add(1);
                if start.saturating_add(len) > buf.len() {
                    return Err(io::ErrorKind::ArgumentListTooLong.into());
                }
                // SAFETY: pointer is valid because we got it from a reference,
                // and we've checked that the buffer has enough size.
                unsafe { mips_argn(i, buf.as_mut_ptr().add(start).cast::<u8>())? }
                start += len;
            }
            Ok((argc, start))
        }
        pub(crate) fn args_bytes<const BUF_SIZE: usize>() -> io::Result<ArgsBytes<BUF_SIZE>> {
            let mut buf = ArgsBytes::<BUF_SIZE>::UNINIT_BUF;
            let (argc, size) = read_args(&mut buf)?;
            Ok(ArgsBytes {
                buf,
                next: Cell::new(0),
                size,
                next_fn: if argc == 1 { next_from_cmdline } else { next_from_args },
            })
        }
        pub(crate) fn args_bytes_in(buf: &mut [MaybeUninit<u8>]) -> io::Result<ArgsBytesRef<'_>> {
            let (argc, size) = read_args(buf)?;
            // SAFETY: read_args initialized buf[..size].
            let buf = unsafe { slice_assume_init_ref(buf.get_unchecked(..size)) };
            Ok(ArgsBytesRef {
                buf,
                next: Cell::new(0),
                next_fn: if argc == 1 { next_from_cmdline } else { next_from_args },
            })
        }
        #[inline]
        pub(crate) fn next<const BUF_SIZE: usize>(args: &ArgsBytes<BUF_SIZE>) -> Option<&[u8]> {
            (args.next_fn)(args.init(), &args.next)
        }
        #[inline]
        pub(crate) fn next_ref<'a>(args: &ArgsBytesRef<'a>) -> Option<&'a [u8]> {
            (args.next_fn)(args.buf, &args.next)
        }
    }
    #[cfg(else)]
    {
        pub(crate) fn args_bytes<const BUF_SIZE: usize>() -> io::Result<ArgsBytes<BUF_SIZE>> {
            Err(io::ErrorKind::Unsupported.into())
        }
        pub(crate) fn args_bytes_in(_buf: &mut [MaybeUninit<u8>]) -> io::Result<ArgsBytesRef<'_>> {
            Err(io::ErrorKind::Unsupported.into())
        }
        pub(crate) fn next<const BUF_SIZE: usize>(_args: &ArgsBytes<BUF_SIZE>) -> Option<&[u8]> {
            unreachable!()
        }
        pub(crate) fn next_ref<'a>(_args: &ArgsBytesRef<'a>) -> Option<&'a [u8]> {
            unreachable!()
        }
    }
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_from_cmdline_length_excludes_nul() {
        let (buf, next) = (&b""[..], Cell::new(0));
        assert_eq!(next_from_cmdline(buf, &next), None);

        let (buf, next) = (&b"prog"[..], Cell::new(0));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"prog"[..]));
        assert_eq!(next_from_cmdline(buf, &next), None);

        let (buf, next) = (&b"prog arg1 arg2"[..], Cell::new(0));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"prog"[..]));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"arg1"[..]));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"arg2"[..]));
        assert_eq!(next_from_cmdline(buf, &next), None);

        let (buf, next) = (&b"prog 'arg 1' \"arg 2\""[..], Cell::new(0));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"prog"[..]));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"arg 1"[..]));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"arg 2"[..]));
        assert_eq!(next_from_cmdline(buf, &next), None);
    }

    #[test]
    fn test_next_from_cmdline_length_includes_nul() {
        let (buf, next) = (&b"\0"[..], Cell::new(0));
        assert_eq!(next_from_cmdline(buf, &next), None);

        let (buf, next) = (&b"prog\0"[..], Cell::new(0));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"prog"[..]));
        assert_eq!(next_from_cmdline(buf, &next), None);

        let (buf, next) = (&b"prog arg1 arg2\0"[..], Cell::new(0));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"prog"[..]));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"arg1"[..]));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"arg2"[..]));
        assert_eq!(next_from_cmdline(buf, &next), None);

        let (buf, next) = (&b"prog 'arg 1' \"arg 2\"\0"[..], Cell::new(0));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"prog"[..]));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"arg 1"[..]));
        assert_eq!(next_from_cmdline(buf, &next), Some(&b"arg 2"[..]));
        assert_eq!(next_from_cmdline(buf, &next), None);
    }
}
