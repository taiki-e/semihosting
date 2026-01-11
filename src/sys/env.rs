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
    next_fn: for<'a> fn(&'a ArgsBytes<BUF_SIZE>) -> Option<&'a [u8]>,
}

impl<const BUF_SIZE: usize> ArgsBytes<BUF_SIZE> {
    const UNINIT_BUF: [MaybeUninit<u8>; BUF_SIZE] = [MaybeUninit::uninit(); BUF_SIZE];
}

pub(crate) fn next_from_cmdline<const BUF_SIZE: usize>(
    args: &ArgsBytes<BUF_SIZE>,
) -> Option<&[u8]> {
    if args.next.get() >= args.size {
        return None;
    }
    // SAFETY: safe due to buf's invariant.
    let buf = unsafe { slice_assume_init_ref(args.buf.get_unchecked(..args.size)) };
    let mut start = args.next.get();
    let mut end = None;
    let is_blank = |b: u8| b == b' ' || b == b'\t';
    let mut delim = NUL;
    let mut in_argument = false;
    while args.next.get() < args.size {
        let b = buf[args.next.get()];
        if !in_argument {
            if is_blank(b) {
                end = Some(args.next.get());
                args.next.set(args.next.get() + 1);
                break;
            }
            if b == b'"' || b == b'\'' {
                delim = b;
                start += 1;
            }
            in_argument = true;
        } else if delim != NUL {
            if b == delim {
                end = Some(args.next.get());
                args.next.set(args.next.get() + 2);
                break;
            }
        } else if is_blank(b) {
            end = Some(args.next.get());
            args.next.set(args.next.get() + 1);
            break;
        }

        args.next.set(args.next.get() + 1);
    }
    Some(&buf[start..end.unwrap_or_else(|| args.next.get())])
}

#[cfg(any(
    test,
    target_arch = "mips",
    target_arch = "mips32r6",
    target_arch = "mips64",
    target_arch = "mips64r6",
))]
#[cfg_attr(test, allow(dead_code))] // TODO(env): unit test
fn next_from_args<const BUF_SIZE: usize>(args: &ArgsBytes<BUF_SIZE>) -> Option<&[u8]> {
    if args.next.get() >= args.size {
        return None;
    }
    // SAFETY: safe due to buf's invariant.
    let buf = unsafe { slice_assume_init_ref(args.buf.get_unchecked(..args.size)) };
    let start = args.next.get();
    let mut end = None;
    while args.next.get() < args.size {
        let b = buf[args.next.get()];
        if b == NUL {
            end = Some(args.next.get());
            args.next.set(args.next.get() + 1);
            break;
        }
        args.next.set(args.next.get() + 1);
    }
    let end = end.unwrap_or_else(|| args.next.get());
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
        all(target_arch = "xtensa", feature = "openocd-semihosting"),
    ))]
    {
        pub(crate) use self::next_from_cmdline as next;
        use crate::sys::arm_compat::sys_get_cmdline_uninit;

        pub(crate) fn args_bytes<const BUF_SIZE: usize>() -> io::Result<ArgsBytes<BUF_SIZE>> {
            let mut buf = ArgsBytes::<BUF_SIZE>::UNINIT_BUF;
            let size = sys_get_cmdline_uninit(&mut buf)?.len();
            Ok(ArgsBytes { buf, next: Cell::new(0), size })
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

        pub(crate) fn args_bytes<const BUF_SIZE: usize>() -> io::Result<ArgsBytes<BUF_SIZE>> {
            let mut buf = ArgsBytes::<BUF_SIZE>::UNINIT_BUF;
            let argc = mips_argc();
            let mut start = 0;
            for i in 0..argc {
                let len = mips_argnlen(i)? + 1;
                if start + len > BUF_SIZE {
                    return Err(io::ErrorKind::ArgumentListTooLong.into());
                }
                // SAFETY: pointer is valid because we got it from a reference,
                // and we've checked that the buffer has enough size.
                unsafe { mips_argn(i, buf.as_mut_ptr().add(start).cast::<u8>())? }
                start += len;
            }
            Ok(ArgsBytes {
                buf,
                next: Cell::new(0),
                size: start,
                next_fn: if argc == 1 { next_from_cmdline } else { next_from_args },
            })
        }
        #[inline]
        pub(crate) fn next<const BUF_SIZE: usize>(args: &ArgsBytes<BUF_SIZE>) -> Option<&[u8]> {
            (args.next_fn)(args)
        }
    }
    #[cfg(else)]
    {
        pub(crate) fn args_bytes<const BUF_SIZE: usize>() -> io::Result<ArgsBytes<BUF_SIZE>> {
            Err(io::ErrorKind::Unsupported.into())
        }
        pub(crate) fn next<const BUF_SIZE: usize>(_args: &ArgsBytes<BUF_SIZE>) -> Option<&[u8]> {
            unreachable!()
        }
    }
});
