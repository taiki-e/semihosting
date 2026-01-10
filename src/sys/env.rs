// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::cell::Cell;

use crate::io;

const NUL: u8 = b'\0';

pub(crate) struct ArgsBytes<const BUF_SIZE: usize> {
    buf: [u8; BUF_SIZE],
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

pub(crate) fn next_from_cmdline<'a, const BUF_SIZE: usize>(
    args: &'a ArgsBytes<BUF_SIZE>,
) -> Option<&'a [u8]> {
    if args.next.get() >= args.size {
        return None;
    }
    let mut start = args.next.get();
    let mut end = None;
    let is_blank = |b: u8| b == b' ' || b == b'\t';
    let mut delim = NUL;
    let mut in_argument = false;
    while args.next.get() < args.size {
        let b = args.buf[args.next.get()];
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
    Some(&args.buf[start..end.unwrap_or_else(|| args.next.get())])
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
        use crate::sys::arm_compat::{CommandLine, sys_get_cmdline};

        pub(crate) fn args_bytes<const BUF_SIZE: usize>() -> io::Result<ArgsBytes<BUF_SIZE>> {
            let mut buf = [0; BUF_SIZE];
            let mut cmdline_block = CommandLine { ptr: buf.as_mut_ptr(), size: BUF_SIZE - 1 };
            unsafe {
                sys_get_cmdline(&mut cmdline_block)?;
            }
            debug_assert!(!cmdline_block.ptr.is_null());
            if cmdline_block.size > BUF_SIZE - 1 || buf[BUF_SIZE - 1] != NUL {
                return Err(io::ErrorKind::ArgumentListTooLong.into());
            }
            Ok(ArgsBytes { buf, next: Cell::new(0), size: cmdline_block.size })
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
            let mut buf = [0; BUF_SIZE];
            let argc = mips_argc();
            let mut start = 0;
            for i in 0..argc {
                let len = mips_argnlen(i)? + 1;
                if start + len > BUF_SIZE {
                    return Err(io::ErrorKind::ArgumentListTooLong.into());
                }
                unsafe { mips_argn(i, buf.as_mut_ptr().add(start))? }
                start += len;
            }
            Ok(ArgsBytes {
                buf,
                next: Cell::new(0),
                size: start,
                next_fn: if argc == 1 { next_from_cmdline } else { next_from_args },
            })
        }
        fn next_from_args<'a, const BUF_SIZE: usize>(
            args: &'a ArgsBytes<BUF_SIZE>,
        ) -> Option<&'a [u8]> {
            if args.next.get() >= args.size {
                return None;
            }
            let start = args.next.get();
            let mut end = None;
            while args.next.get() < args.size {
                let b = args.buf[args.next.get()];
                if b == NUL {
                    end = Some(args.next.get());
                    args.next.set(args.next.get() + 1);
                    break;
                }
                args.next.set(args.next.get() + 1);
            }
            let end = end.unwrap_or_else(|| args.next.get());
            let last = end.saturating_sub(1);
            if start != last
                && (args.buf[start] == b'"' && args.buf[last] == b'"'
                    || args.buf[start] == b'\'' && args.buf[last] == b'\'')
            {
                Some(&args.buf[start + 1..last])
            } else {
                Some(&args.buf[start..end])
            }
        }
        #[inline]
        pub(crate) fn next<'a, const BUF_SIZE: usize>(
            args: &'a ArgsBytes<BUF_SIZE>,
        ) -> Option<&'a [u8]> {
            (args.next_fn)(args)
        }
    }
    #[cfg(else)]
    {
        pub(crate) fn args_bytes<const BUF_SIZE: usize>() -> io::Result<ArgsBytes<BUF_SIZE>> {
            Err(io::ErrorKind::Unsupported.into())
        }
        pub(crate) fn next<'a, const BUF_SIZE: usize>(
            _args: &'a ArgsBytes<BUF_SIZE>,
        ) -> Option<&'a [u8]> {
            unreachable!()
        }
    }
});
