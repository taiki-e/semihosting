// SPDX-License-Identifier: Apache-2.0 OR MIT

#![no_main]
#![no_std]
#![warn(unsafe_op_in_unsafe_fn)]

use core::str;

#[cfg(arm_compat)]
use semihosting::sys::arm_compat::*;
#[cfg(mips)]
use semihosting::sys::mips::*;
use semihosting::{
    c, dbg, experimental,
    fd::AsFd,
    fs,
    io::{self, IsTerminal, Read, Seek, Write},
    print, println,
};

semihosting_no_std_test_rt::entry!(run_main);
#[cfg(feature = "panic-unwind")]
fn run_main() -> i32 {
    unsafe { allocator::init_global_allocator() }
    match experimental::panic::catch_unwind(run) {
        Ok(()) => 0,
        Err(_) => 101,
    }
}
#[cfg(not(feature = "panic-unwind"))]
use run as run_main;

fn run() {
    #[cfg(feature = "panic-unwind")]
    {
        #[inline(never)]
        fn a() {
            panic!("a");
        }
        #[inline(never)]
        fn b() {
            a()
        }
        experimental::panic::catch_unwind(|| a()).unwrap_err();
        experimental::panic::catch_unwind(|| b()).unwrap_err();
    }

    // TODO
    if cfg!(armv4t) && cfg!(feature = "qemu-system")
        || cfg!(all(target_arch = "arm", target_endian = "big")) && cfg!(feature = "qemu-system")
    {
        if cfg!(armv4t) {
        } else {
            println!("this message does not print...");
            io::stdout().unwrap_err();
            io::stderr().unwrap_err();
        }
        return;
    }

    let stdio_is_terminal = option_env!("CI").is_none() || cfg!(mips) && !cfg!(host_linux);
    // TODO: return result?
    #[cfg(not(mips))]
    let now = experimental::time::SystemTime::now();
    {
        print!("test io ... ");
        // TODO
        // assert_eq!(core::mem::size_of::<io::Error>(), core::mem::size_of::<u64>());
        // assert_eq!(core::mem::size_of::<io::Result<()>(), core::mem::size_of::<u64>());
        println!("ok");
    }
    {
        print!("test io::stdio ... ");
        let mut stdout1 = io::stdout().unwrap();
        let mut stdout2 = io::stdout().unwrap();
        let mut stderr1 = io::stderr().unwrap();
        let mut stderr2 = io::stderr().unwrap();
        println!("stdout1: {}", stdout1.as_fd().as_raw_fd());
        println!("stdout2: {}", stdout2.as_fd().as_raw_fd());
        println!("stderr1: {}", stderr1.as_fd().as_raw_fd());
        println!("stderr2: {}", stderr2.as_fd().as_raw_fd());
        #[cfg(mips)]
        {
            assert_eq!(stdout1.as_fd().as_raw_fd(), 1);
            assert_eq!(stdout2.as_fd().as_raw_fd(), 1);
            assert_eq!(stderr1.as_fd().as_raw_fd(), 2);
            assert_eq!(stderr2.as_fd().as_raw_fd(), 2);
            assert_eq!(
                mips_open(c!("/dev/stdout"), O_WRONLY, 0o666).unwrap().as_fd().as_raw_fd(),
                1
            );
            assert_eq!(
                mips_open(c!("/dev/stderr"), O_WRONLY, 0o666).unwrap().as_fd().as_raw_fd(),
                2
            );
            assert_eq!(
                mips_open(c!("/dev/stdout"), O_WRONLY, 0o666).unwrap().as_fd().as_raw_fd(),
                mips_open(c!("/dev/stdout"), O_WRONLY, 0o666).unwrap().as_fd().as_raw_fd(),
            );
            assert_eq!(
                mips_open(c!("/dev/stderr"), O_WRONLY, 0o666).unwrap().as_fd().as_raw_fd(),
                mips_open(c!("/dev/stderr"), O_WRONLY, 0o666).unwrap().as_fd().as_raw_fd(),
            );
        }
        #[cfg(not(mips))]
        {
            assert_ne!(stdout1.as_fd().as_raw_fd(), stdout2.as_fd().as_raw_fd());
            assert_ne!(stderr1.as_fd().as_raw_fd(), stderr2.as_fd().as_raw_fd());
        }
        assert_eq!(stdout1.is_terminal(), stdio_is_terminal);
        assert_eq!(stderr1.is_terminal(), stdio_is_terminal);
        stdout1.write_all(b"hello\n").unwrap();
        stdout2.write_all(b"hello\n").unwrap();
        stderr1.write_all(b"world\n").unwrap();
        stderr2.write_all(b"world\n").unwrap();
        drop(stdout1);
        drop(stdout2);
        drop(stderr1);
        drop(stderr2);
        let f1 = io::stdout().unwrap().as_fd().as_raw_fd();
        assert_eq!(io::stdout().unwrap().as_fd().as_raw_fd(), f1);

        if cfg!(mips) {
            let stdin = io::stdin().unwrap();
            assert_eq!(stdin.as_fd().as_raw_fd(), 0);
            assert_eq!(stdin.is_terminal(), stdio_is_terminal);
        } else {
            let mut stdin = io::stdin().unwrap();
            // in tests, we use <<< to input stdin, so stdin is not tty.
            // assert_eq!(stdin.is_terminal(), stdio_is_terminal);
            assert_eq!(stdin.is_terminal(), false);
            let mut buf = [0; 3];
            let n = stdin.read(&mut buf[..]).unwrap();
            assert_eq!(n, 3);
            let s = str::from_utf8(&buf[..n]).unwrap();
            assert_eq!(s, "std");
            let n = stdin.read(&mut buf[..]).unwrap();
            assert_eq!(n, 3);
            let s = str::from_utf8(&buf[..n]).unwrap();
            assert_eq!(s, "in\n");
        }
        dbg!(());
        println!("ok");
    }
    {
        print!("test fs ... ");
        let check_metadata = option_env!("CI").is_none() || cfg!(not(mips));
        let path_a = c!("a.txt");
        let path_b = c!("b.txt");
        // create/write/seek
        let mut file = fs::File::create(path_a).unwrap();
        assert_eq!(file.is_terminal(), false);
        if check_metadata {
            assert_eq!(file.metadata().unwrap().len(), 0);
        }
        file.write_all(b"abb").unwrap();
        if check_metadata {
            assert_eq!(file.metadata().unwrap().len(), 3);
        }
        assert_eq!(file.seek(io::SeekFrom::Start(2)).unwrap(), 2);
        file.write_all(b"c").unwrap();
        if check_metadata {
            assert_eq!(file.metadata().unwrap().len(), 3);
        }
        assert_eq!(file.seek(io::SeekFrom::Start(2)).unwrap(), 2);
        if check_metadata {
            assert_eq!(file.metadata().unwrap().len(), 3);
        }
        assert_eq!(file.seek(io::SeekFrom::Start(100)).unwrap(), 100);
        if check_metadata {
            assert_eq!(file.metadata().unwrap().len(), 3);
        }
        assert_eq!(file.seek(io::SeekFrom::Start(2)).unwrap(), 2);
        if check_metadata {
            assert_eq!(file.metadata().unwrap().len(), 3);
        }
        file.write_all(b"cde").unwrap();
        if check_metadata {
            assert_eq!(file.metadata().unwrap().len(), 5);
        }
        let mut buf = [0; 4];
        if cfg!(mips) {
            let errno = file.read(&mut buf[..]).unwrap_err().raw_os_error().unwrap();
            assert!(errno == 22 || errno == 9, "{}", errno);
        } else {
            // TODO: if no read permission, arm semihosting handles it like eof.
            assert_eq!(file.read(&mut buf[..]).unwrap(), 0);
        }
        assert_eq!(
            file.seek(io::SeekFrom::End(-200)).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(
            file.seek(io::SeekFrom::Start(usize::MAX as _)).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        drop(file);

        // open/read/seek
        let mut buf = [0; 4];
        let mut file = fs::File::open(path_a).unwrap();
        file.write_all(b"a").unwrap_err(); // no write permission
        if check_metadata {
            assert_eq!(file.metadata().unwrap().len(), 5);
        }
        let n = file.read(&mut buf[..]).unwrap();
        assert_eq!(n, 4);
        let s = str::from_utf8(&buf[..n]).unwrap();
        assert_eq!(s, "abcd");
        let n = file.read(&mut buf[..]).unwrap();
        assert_eq!(n, 1);
        assert_eq!(str::from_utf8(&buf[..n]).unwrap(), "e");
        assert_eq!(file.seek(io::SeekFrom::Start(3)).unwrap(), 3);
        let n = file.read(&mut buf[..]).unwrap();
        assert_eq!(n, 2);
        assert_eq!(str::from_utf8(&buf[..n]).unwrap(), "de");
        drop(file);

        // rename
        if cfg!(mips) {
            assert_eq!(fs::rename(path_a, path_b).unwrap_err().kind(), io::ErrorKind::Unsupported);
        } else {
            fs::rename(path_a, path_b).unwrap();
            assert_eq!(fs::File::open(path_a).unwrap_err().kind(), io::ErrorKind::NotFound);
            let mut file = fs::File::open(path_b).unwrap();
            let mut buf = [0; 8];
            let n = file.read(&mut buf[..]).unwrap();
            assert_eq!(n, 5);
            assert_eq!(str::from_utf8(&buf[..n]).unwrap(), "abcde");
            drop(file);
            fs::rename(path_b, path_a).unwrap();
            fs::File::open(path_a).unwrap();
            assert_eq!(fs::File::open(path_b).unwrap_err().kind(), io::ErrorKind::NotFound);
        }

        fs::remove_file(path_a).unwrap();
        println!("ok");
    }
    {
        println!("test env::args ... ");
        const BUF_SIZE: usize = 128;
        #[cfg(arm_compat)]
        {
            let mut buf = [0; BUF_SIZE];
            let mut cmdline_block = CommandLine { ptr: buf.as_mut_ptr(), size: BUF_SIZE - 1 };
            unsafe {
                sys_get_cmdline(&mut cmdline_block).unwrap();
                println!("args '{}'", str::from_utf8(&buf[..cmdline_block.size]).unwrap());
            }
        }
        let args = experimental::env::args::<BUF_SIZE>().unwrap();
        println!("arg0: '{}'", (&args).next().unwrap().unwrap());
        assert_eq!((&args).next().unwrap().unwrap(), "a");
        assert_eq!((&args).next().unwrap().unwrap(), "");
        assert_eq!((&args).next().unwrap().unwrap(), "c d");
        assert_eq!((&args).next(), None);
        println!("ok");
    }
    #[cfg(arm_compat)]
    {
        // sys_*
        println!("sys_clock: {}", sys_clock().unwrap());
        if cfg!(not(all(target_arch = "arm", target_feature = "v8", target_feature = "rclass"))) {
            println!("sys_elapsed: {}", sys_elapsed().unwrap());
        }
        // TODO: sys_heapinfo
        assert_eq!(sys_iserror(isize::MAX), false);
        assert_eq!(sys_iserror(1), false);
        assert_eq!(sys_iserror(0), false);
        assert_eq!(sys_iserror(-1), true);
        assert_eq!(sys_iserror(-4095), true);
        assert_eq!(sys_iserror(-4096), true);
        assert_eq!(sys_iserror(isize::MIN), true);
        // println!("{}", sys_readc() as char); // only works on qemu-user
        println!("sys_system: {}", sys_system(c!("pwd")));
        println!("sys_tickfreq: {}", sys_tickfreq().unwrap());
        println!("sys_time: {}", sys_time().unwrap());
        print!("sys_writec: ");
        sys_writec(b'a');
        sys_writec(b'\n');
        print!("sys_write0: ");
        sys_write0(c!("bc\n"));
    }
    #[cfg(mips)]
    {}

    #[cfg(not(mips))]
    println!("elapsed: {:?}", now.elapsed().unwrap());
}

#[cfg(feature = "panic-unwind")]
mod allocator {
    use core::{cell::UnsafeCell, mem::MaybeUninit};
    // linked_list_allocator's LockedHeap uses spinning_top, but it doesn't compatible
    // with targets without atomic CAS. Implement our own LockedHeap by using spin,
    // which supports portable-atomic.
    #[global_allocator]
    static ALLOCATOR: LockedHeap =
        LockedHeap(spin::Mutex::new(linked_list_allocator::Heap::empty()));
    #[inline(always)]
    pub unsafe fn init_global_allocator() {
        const HEAP_SIZE: usize = 1024;
        static HEAP_MEM: SyncUnsafeCell<[MaybeUninit<u8>; HEAP_SIZE]> =
            SyncUnsafeCell::new([MaybeUninit::uninit(); HEAP_SIZE]);
        unsafe { ALLOCATOR.0.lock().init(HEAP_MEM.get().cast::<u8>(), HEAP_SIZE) }
    }
    struct LockedHeap(spin::Mutex<linked_list_allocator::Heap>);
    unsafe impl core::alloc::GlobalAlloc for LockedHeap {
        unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
            self.0
                .lock()
                .allocate_first_fit(layout)
                .ok()
                .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
            unsafe { self.0.lock().deallocate(core::ptr::NonNull::new_unchecked(ptr), layout) }
        }
    }
    // See https://github.com/rust-lang/rust/issues/53639
    #[repr(transparent)]
    struct SyncUnsafeCell<T: ?Sized> {
        value: UnsafeCell<T>,
    }
    unsafe impl<T: ?Sized> Sync for SyncUnsafeCell<T> {}
    impl<T> SyncUnsafeCell<T> {
        #[inline]
        const fn new(value: T) -> Self {
            Self { value: UnsafeCell::new(value) }
        }
        #[inline]
        const fn get(&self) -> *mut T {
            self.value.get()
        }
    }
}
