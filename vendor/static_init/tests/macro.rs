// Copyright 2021 Olivier Kannengieser
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
#![cfg_attr(feature = "test_thread_local", feature(thread_local))]

extern crate static_init;
use static_init::{constructor, destructor, dynamic};

static mut DEST: i32 = 0;

#[destructor(0)]
extern "C" fn dest_0() {
    unsafe {
        assert_eq!(DEST, 0);
        DEST += 1;
    }
}

#[destructor(1)]
extern "C" fn dest_1() {
    unsafe {
        assert_eq!(DEST, 1);
        DEST += 1;
    }
}
#[destructor(100)]
extern "C" fn dest_2() {
    unsafe {
        assert_eq!(DEST, 2);
        DEST += 1;
    }
}

static mut INI: i32 = 0;

#[constructor(200)]
extern "C" fn init_2() {
    unsafe {
        assert_eq!(INI, 0);
        INI += 1;
    }
}
#[constructor(1)]
extern "C" fn init_1() {
    unsafe {
        assert_eq!(INI, 1);
        INI += 1;
    }
}
#[constructor(0)]
extern "C" fn init_0() {
    unsafe {
        assert_eq!(INI, 2);
        INI += 1;
    }
}

#[cfg(all(unix, target_env = "gnu"))]
mod gnu {
    use super::constructor;
    use std::env::args_os;
    use std::ffi::{CStr, OsStr};
    use std::os::unix::ffi::OsStrExt;

    #[constructor]
    extern "C" fn get_args_env(argc: i32, mut argv: *const *const u8, _env: *const *const u8) {
        let mut argc_counted = 0;
        unsafe {
            while !(*argv).is_null() {
                assert!(
                    args_os()
                        .any(|x| x
                            == OsStr::from_bytes(CStr::from_ptr(*argv as *const i8).to_bytes()))
                );
                argv = argv.add(1);
                argc_counted += 1
            }
        }
        assert_eq!(argc_counted, argc);
    }
}

#[derive(Debug, Eq, PartialEq)]
struct A(i32);

impl A {
    fn new(v: i32) -> A {
        A(v)
    }
}
impl Drop for A {
    fn drop(&mut self) {
        assert_eq!(self.0, 33)
    }
}

#[test]
fn inner_static() {
    #[dynamic(0)]
    static IX: usize = unsafe { &IX as *const _ as usize };
    #[dynamic(0)]
    static IX2: usize = unsafe { &IX2 as *const _ as usize };

    static mut I: i32 = 0;

    #[constructor]
    extern "C" fn f() {
        unsafe { I = 3 }
    }

    unsafe {
        assert_eq!(*IX, &IX as *const _ as usize);
        assert_eq!(*IX2, &IX2 as *const _ as usize);
        assert_eq!(I, 3)
    };
}

#[dynamic(0)]
static mut V0: A = A::new(unsafe { V1.0 } - 5);

#[dynamic(20)]
static mut V2: A = A::new(12);

#[dynamic(10)]
static V1: A = A::new(unsafe { V2.0 } - 2);

#[dynamic(init = 20)]
static mut V3: A = A::new(12);

#[dynamic(init = 10)]
static V4: A = A::new(unsafe { V2.0 } - 2);

#[dynamic(init = 5, drop = 5)]
static V5: A = A::new(unsafe { V4.0 } + 23);

#[dynamic(drop_only = 0)]
static V6: A = A(33);

#[test]
fn dynamic_init() {
    unsafe {
        assert_eq!(V0.0, 5);
        assert_eq!(V1.0, 10);
        assert_eq!(V2.0, 12);
        V2.0 = 8;
        assert_eq!(V2.0, 8);
        assert_eq!(V4.0, 10);
        assert_eq!(V3.0, 12);
        assert_eq!(V5.0, 33);
        assert_eq!(V6.0, 33);
    }
}


#[cfg(feature = "atexit")]
mod atexit {
    use static_init::{destructor, dynamic};

    static mut DROP_V: i32 = 0;

    struct C(i32);

    impl Drop for C {
        fn drop(&mut self) {
            unsafe {
                assert_eq!(self.0, DROP_V);
                DROP_V += 1;
            };
        }
    }

    #[dynamic(init, drop)]
    static C3: C = C(0);

    #[dynamic(10, drop)]
    static C2: C = C(1);

    #[dynamic(20, drop)]
    static C1: C = C(2);

    #[destructor]
    extern "C" fn check_drop_v() {
        unsafe { assert_eq!(DROP_V, 3) }
    }
}

#[cfg(feature = "lazy")]
mod lazy {

    #[cfg(feature = "test_thread_local")]
    #[test]
    fn thread_local() {
        #[thread_local]
        #[dynamic(lazy)]
        static mut TH_LOCAL: A = A::new(3);

        unsafe {
            assert_eq!(TH_LOCAL.0, 3);
            TH_LOCAL.0 = 42;
            assert_eq!(TH_LOCAL.0, 42);
        }
        std::thread::spawn(|| unsafe {
            assert_eq!(TH_LOCAL.0, 3);
        })
        .join()
        .unwrap();
    }

    #[cfg(all(feature= "thread_local_drop",feature = "test_thread_local"))]
    #[test]
    fn thread_local_drop() {
        use core::sync::atomic::{AtomicI32, Ordering};
        #[thread_local]
        #[dynamic(lazy, drop)]
        static TH_LOCAL_UNSAFE: i32 = 10;

        assert_eq!(unsafe { *TH_LOCAL_UNSAFE }, 10);

        static DROP_COUNT: AtomicI32 = AtomicI32::new(0);

        struct B;

        impl Drop for B {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::Relaxed);
            }
        }

        #[thread_local]
        #[dynamic(lazy, drop)]
        static B1: B = B;

        #[thread_local]
        #[dynamic(lazy, drop)]
        static mut B2: B = B;

        std::thread::spawn(|| unsafe {
            &*B1;
            &*B2
        })
        .join()
        .unwrap();
        std::thread::spawn(|| ()).join().unwrap();
        std::thread::spawn(|| unsafe {
            &*B1;
            &*B2
        })
        .join()
        .unwrap();
        assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 4);
    }

    use super::A;
    use static_init::dynamic;
    #[dynamic(lazy)]
    static L1: A = A::new(unsafe { L0.0 } + 1);

    #[dynamic(lazy)]
    static mut L0: A = A::new(10);

    #[cfg(feature = "atexit")]
    #[dynamic(lazy, drop)]
    static L3: A = A::new(33);

    #[cfg(feature = "atexit")]
    #[dynamic(lazy, drop)]
    static mut L2: A = A::new(unsafe { L3.0 });

    #[test]
    fn lazy_init() {
        unsafe { assert_eq!(L0.0, 10) };
        assert_eq!(L1.0, 11);
    }
}
