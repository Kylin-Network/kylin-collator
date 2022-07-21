// Copyright 2021 Olivier Kannengieser 
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![cfg_attr(feature = "test_thread_local",feature(thread_local))]

#[cfg(all(feature = "test_thread_local", debug_mode))]
mod test {
    use static_init::{constructor, dynamic};

    #[thread_local]
    #[dynamic(lazy)]
    static mut V0: i32 = unsafe{*V0};

    fn panic_hook(p: &core::panic::PanicInfo<'_>) -> () {
        println!("Panic caught {}", p);
        std::process::exit(0)
    }

    #[constructor(10)]
    extern "C" fn set_hook() {
        std::panic::set_hook(Box::new(panic_hook));
    }
}

fn panic_hook(p: &core::panic::PanicInfo<'_>) -> () {
    println!("Panic caught {}", p);
    std::process::exit(1)
}

#[test]
fn bad_cyclic_thread_local_lazy() {
    std::panic::set_hook(Box::new(panic_hook));
    assert!(!cfg!(all(debug_mode,feature = "test_thread_local")));
}
