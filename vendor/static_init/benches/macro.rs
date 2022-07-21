// Copyright 2021 Olivier Kannengieser 
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![feature(test)]
extern crate static_init;
use ctor::ctor;
use static_init::dynamic;

extern crate test;
use std::sync::atomic::{AtomicI32, Ordering};
use test::Bencher;

extern crate lazy_static;
use lazy_static::lazy_static;

static O: AtomicI32 = AtomicI32::new(0);

static mut O_: i32 = 0;

#[dynamic(10)]
static W: AtomicI32 = AtomicI32::new(0);

#[dynamic(10)]
static mut W_: i32 = 0;

#[dynamic(10)]
static mut WM: AtomicI32 = AtomicI32::new(0);

lazy_static! {
    static ref WL: AtomicI32 = AtomicI32::new(0);
    static ref WL1: AtomicI32 = AtomicI32::new(WL2.load(Ordering::Relaxed));
    static ref WL2: AtomicI32 = AtomicI32::new(WL1.load(Ordering::Relaxed));
}

#[ctor]
static WCT: AtomicI32 = AtomicI32::new(0);

#[dynamic(lazy)]
static L: AtomicI32 = AtomicI32::new(0);
#[dynamic(lazy)]
static mut L_: i32 = 0;


#[bench]
fn atomic_regular(bench: &mut Bencher) {
    bench.iter(|| O.fetch_add(1, Ordering::Relaxed));
}
#[bench]
fn atomic_dynamic_static(bench: &mut Bencher) {
    bench.iter(|| unsafe{W.fetch_add(1, Ordering::Relaxed)});
}
#[bench]
fn atomic_lesser_lazy_static(bench: &mut Bencher) {
    bench.iter(|| L.fetch_add(1, Ordering::Relaxed));
}
#[bench]
fn atomic_lazy_static_crate(bench: &mut Bencher) {
    bench.iter(|| WL.fetch_add(1, Ordering::Relaxed));
}
#[bench]
fn atomic_ctor_crate(bench: &mut Bencher) {
    bench.iter(|| WCT.fetch_add(1, Ordering::Relaxed));
}


#[bench]
fn regular(bench: &mut Bencher) {
    bench.iter(|| unsafe{O_+=1});
}

#[bench]
fn dynamic_static(bench: &mut Bencher) {
    bench.iter(|| unsafe{*W_+=1});
}
#[bench]
fn atomic_dynamic_static_mutable(bench: &mut Bencher) {
    bench.iter(|| unsafe { WM.fetch_add(1, Ordering::Relaxed) });
}
#[bench]
fn lesser_lazy_static(bench: &mut Bencher) {
    bench.iter(|| unsafe{*L_+=1});
}
//access to lazy static cost 2ns
