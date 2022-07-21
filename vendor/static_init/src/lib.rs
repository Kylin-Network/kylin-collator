// Copyright 2021 Olivier Kannengieser
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![cfg_attr(not(any(feature = "lazy",feature="thread_local_drop")), no_std)]
//! Non const static initialization, and program constructor/destructor code.
//!
//! # Lesser Lazy Statics
//!
//! This crate provides *lazy statics* on all plateforms.
//!
//! On unixes and windows *lesser lazy statics* are *lazy* during program startup phase
//! (before `main` is called). Once main is called, those statics are all guaranteed to be
//! initialized and any access to them is as fast as any access to regular const initialized
//! statics. Benches sho that usual lazy statics, as those provided by `std::lazy::*` or from
//! [lazy_static][1] crate, suffer from a 2ns access penalty.
//!
//! *Lesser lazy statics* can optionaly be dropped at program destruction
//! (after main exit but before the program stops).
//!
//! *Lesser lazy statics* require the standard library and are enabled by default
//! crate features `lazy` and `atexit`.
//! ```rust
//! use static_init::{dynamic};
//!
//! #[dynamic] //equivalent to #[dynamic(lazy)]
//! static L1: Vec<i32> = unsafe{L0.clone()};
//!
//! #[dynamic(drop)] //equivalent to #[dynamic(lazy,drop)]
//! static L0: Vec<i32> = vec![1,2,3];
//!
//! #[dynamic(drop)]
//! static mut L2: Vec<i32> = L1.clone();
//! #
//! # assert_eq!(L1[0], 1);
//! # unsafe {
//! #     assert_eq!(L2[1], 2);
//! #     L2[1] = 42;
//! #     assert_eq!(L2[1], 42);
//! #     }
//! #     
//! ```
//! As can be seen above accesses to *lazy static* that are dropped must be within unsafe
//! blocks. The reason is that it is possible at program destruction to access already dropped
//! lazy statics.
//!
//! # Dynamic statics: statics initialized at program startup
//!
//! On plateforms that support it (unixes, mac, windows), this crate provides *dynamic statics*: statics that are
//! initialized at program startup. This feature is `no_std`.
//!
//! ```rust
//! use static_init::{dynamic};
//!
//! #[dynamic(0)]
//! //equivalent to #[dynamic(init=0)]
//! static D1: Vec<i32> = vec![1,2,3];
//! 
//! assert_eq!(unsafe{D1[0]}, 1);
//! ```
//! As can be seen above, even if D1 is not mutable, access to it must be performed in unsafe
//! blocks. The reason is that during startup phase, accesses to *dynamic statics* may cause
//! *undefined behavior*: *dynamic statics* may be in a zero initialized state.
//!
//! To prevent such hazardeous accesses, on unixes and window plateforms, a priority can be
//! specified. Dynamic static initializations with higher priority are sequenced before dynamic
//! static initializations with lower priority. Dynamic static initializations with the same
//! priority are underterminately sequenced.
//!
//! ```rust
//! use static_init::{dynamic};
//!
//! // D2 initialization is sequenced before D1 initialization
//! #[dynamic(0)]
//! static mut D1: Vec<i32> = unsafe{D2.clone()};
//!
//! #[dynamic(10)]
//! static D2: Vec<i32> = vec![1,2,3];
//! #
//! # unsafe{assert_eq!(D1[0], 1)};
//! ```
//!
//! *Dynamic statics* can be dropped at program destruction phase: they are dropped after main
//! exit:
//!
//! ```rust
//! use static_init::{dynamic};
//!
//! // D2 initialization is sequenced before D1 initialization
//! // D1 drop is sequenced before D2 drop.
//! #[dynamic(init=0,drop=0)]
//! static mut D1: Vec<i32> = unsafe {D2.clone()};
//!
//! #[dynamic(init=10,drop=10)]
//! static D2: Vec<i32> = vec![1,2,3];
//! ```
//! The priority act on drop in reverse order. *Dynamic statics* drops with a lower priority are
//! sequenced before *dynamic statics* drops with higher priority.
//!
//! Finally, if the feature `atexit` is enabled, *dynamic statics* drop can be registered with
//! `libc::atexit`. *lazy dynamic statics* and *dynamic statics* with `drop_reverse` attribute
//! argument are destroyed in the reverse order of their construction. Functions registered with
//! `atexit` are executed before program destructors and drop of *dynamic statics* that use the
//! `drop` attribute argument. Drop is registered with at `atexit` if no priority if given to the
//! `drop` attribute argument.
//!
//! ```rust
//! use static_init::{dynamic};
//!
//! //D1 is dropped before D2 because
//! //it is initialized before D2
//! #[dynamic(lazy,drop)]
//! static D1: Vec<i32> = vec![0,1,2];
//!
//! #[dynamic(10,drop)]
//! static D2: Vec<i32> = unsafe{D1.clone()};
//!
//! //D3 is initilized after D1 and D2 initializations
//! //and it is dropped after D1 and D2 drops
//! #[dynamic(5,drop)]
//! static D3: Vec<i32> = unsafe{D1.clone()};
//! ```
//!
//! # Constructor and Destructor
//!
//! On plateforms that support it (unixes, mac, windows), this crate provides a way to declare
//! *constructors*: a function called before main is called. This feature is `no_std`.
//!
//! ```rust
//! use static_init::{constructor};
//!
//! //called before main
//! #[constructor] //equivalent to #[constructor(0)]
//! extern "C" fn some_init() {}
//! ```
//!
//! Constructors also support priorities. Sequencement rules applies also between constructor calls and
//! between *dynamic statics* initialization and *constructor* calls.
//!
//! *destructors* are called at program destruction. They also support priorities.
//!
//! ```rust
//! use static_init::{constructor, destructor};
//!
//! //called before some_init
//! #[constructor(10)]
//! extern "C" fn pre_init() {}
//!
//! //called before main
//! #[constructor]
//! extern "C" fn some_init() {}
//!
//! //called after main
//! #[destructor]
//! extern "C" fn first_destructor() {}
//!
//! //called after first_destructor
//! #[destructor(10)]
//! extern "C" fn last_destructor() {}
//! ```
//!
//! # Thread Local Support
//!
//! Variable declared with `#[dynamic(lazy)]` can also be declared `#[thread_local]`. These
//! variable will behave as regular *lazy statics*.
//! ```ignore
//! #[thread_local]
//! #[dynamic(lazy)]
//! static mut X: Vec<i32> = vec![1,2,3];
//! ```
//! These variables can also be droped on thread exit.
//! ```ignore
//! #[thread_local]
//! #[dynamic(lazy,drop)]
//! static X: Vec<i32> = vec![1,2,3];
//!
//! assert!(unsafe{X[1] == 2});
//! ```
//!
//! Accessing a thread local *lazy statics* that should drop during the phase where thread_locals are
//! droped may cause *undefined behavior*. For this reason any access to a thread local lazy static
//! that is dropped will require an unsafe block, even if the static is const.
//!
//!
//! # Debuging initialization order
//!
//! If the feature `debug_order` is enabled, attempts to access `dynamic statics` that are
//! uninitialized or whose initialization is undeterminately sequenced with the access will cause
//! a panic with a message specifying which statics was tentatively accessed and how to change this
//! *dynamic static* priority to fix this issue.
//!
//! Run `cargo test` in this crate directory to see message examples.
//!
//! All implementations of lazy statics may suffer from circular initialization dependencies. Those
//! circular dependencies will cause either a dead lock or an infinite loop. If the feature `debug_order` is
//! enabled, atemp are made to detect those circular dependencies. In most case they will be detected.
//!
//! [1]: https://crates.io/crates/lazy_static

#[doc(hidden)]
/// # Details and implementation documentation.
///
/// ## Mac
///   - [MACH_O specification](https://www.cnblogs.com/sunkang/archive/2011/05/24/2055635.html)
///   - GCC source code gcc/config/darwin.c indicates that priorities are not supported.
///
///   Initialization functions pointers are placed in section "__DATA,__mod_init_func" and
///   "__DATA,__mod_term_func"
///
///   std::env is not initialized in any constructor.
///
/// ## ELF plateforms:
///  - `info ld`
///  - linker script: `ld --verbose`
///  - [ELF specification](https://docs.oracle.com/cd/E23824_01/html/819-0690/chapter7-1.html#scrolltoc)
///
///  The runtime will run fonctions pointers of section ".init_array" at startup and function
///  pointers in ".fini_array" at program exit. The linker place in the target object file
///  sectio .init_array all sections from the source objects whose name is of the form
///  .init_array.NNNNN in lexicographical order then the .init_array sections of those same source
///  objects. It does equivalently with .fini_array and .fini_array.NNNN sections.
///
///  Usage can be seen in gcc source gcc/config/pru.c
///
///  Resources of libstdc++ are initialized with priority 65535-100 (see gcc source libstdc++-v3/c++17/default_resource.h)
///  The rust standard library function that capture the environment and executable arguments is
///  executed at priority 65535-99 on gnu platform variants. On other elf plateform they are not accessbile in any constructors. Nevertheless
///  one can read into /proc/self directory to retrieve the command line.
///  Some callbacks constructors and destructors with priority 65535 are
///  registered by rust/rtlibrary.
///  Static C++ objects are usually initialized with no priority (TBC). lib-c resources are
///  initialized by the C-runtime before any function in the init_array (whatever the priority) are executed.
///
/// ## Windows
///
///   std::env is initialized before any constructors.
///
///  - [this blog post](https://www.cnblogs.com/sunkang/archive/2011/05/24/2055635.html)
///
///  At start up, any functions pointer between sections ".CRT$XIA" and ".CRT$XIZ"
///  and then any functions between ".CRT$XCA" and ".CRT$XCZ". It happens that the C library
///  initialization functions pointer are placed in ".CRT$XIU" and C++ statics functions initialization
///  pointers are placed in ".CRT$XCU". At program finish the pointers between sections
///  ".CRT$XPA" and ".CRT$XPZ" are run first then those between ".CRT$XTA" and ".CRT$XTZ".
///
///  Some reverse engineering was necessary to find out a way to implement
///  constructor/destructor priority.
///
///  Contrarily to what is reported in this blog post, msvc linker
///  only performs a lexicographicall ordering of section whose name
///  is of the form "\<prefix\>$\<suffix\>" and have the same \<prefix\>.
///  For example "RUST$01" and "RUST$02" will be ordered but those two
///  sections will not be ordered with "RHUM" section.
///
///  Moreover, it seems that section name of the form \<prefix\>$\<suffix\> are
///  not limited to 8 characters.
///
///  So static initialization function pointers are placed in section ".CRT$XCU" and
///  those with a priority `p` in `format!(".CRT$XCTZ{:05}",65535-p)`. Destructors without priority
///  are placed in ".CRT$XPU" and those with a priority in `format!(".CRT$XPTZ{:05}",65535-p)`.
mod details {}

use core::mem::ManuallyDrop;

#[doc(inline)]
pub use static_init_macro::constructor;

#[doc(inline)]
pub use static_init_macro::destructor;

#[doc(inline)]
pub use static_init_macro::dynamic;

#[cfg(feature = "lazy")]
mod static_lazy;

#[cfg(feature = "lazy")]
pub use static_lazy::{Lazy,ConstLazy};

mod thread_local_lazy;

pub use thread_local_lazy::{Lazy as ThreadLocalLazy, ConstLazy as ThreadLocalConstLazy};

#[cfg(feature = "thread_local_drop")]
pub use thread_local_lazy::__push_tls_destructor;

union StaticBase<T> {
    k: (),
    v: ManuallyDrop<T>,
}

#[derive(Debug)]
#[doc(hidden)]
pub enum InitMode {
    Const,
    Lazy,
    Dynamic(u16),
}

#[derive(Debug)]
#[doc(hidden)]
pub enum DropMode {
    None,
    AtExit,
    Dynamic(u16),
}


#[derive(Debug)]
#[doc(hidden)]
pub struct StaticInfo {
    pub variable_name: &'static str,
    pub file_name:     &'static str,
    pub line:          u32,
    pub column:        u32,
    pub init_mode: InitMode,
    pub drop_mode: DropMode,
}

pub use static_impl::{Static, ConstStatic,__set_init_prio};

#[cfg(debug_mode)]
mod static_impl {
    use super::{StaticBase,StaticInfo,InitMode,DropMode};
    use core::mem::ManuallyDrop;
    use core::ops::{Deref,DerefMut};
    use core::cell::UnsafeCell;
  /// The actual type of mutable *dynamic statics*.
  ///
  /// It implements `Deref<Target=T>` and `DerefMut`.
  ///
  /// All associated functions are only usefull for the implementation of
  /// the `dynamic` proc macro attribute
  pub struct Static<T>(
      StaticBase<T>,
      StaticInfo,
      AtomicI32,
  );

    /// The actual type of non mutable *dynamic statics*.
    ///
    /// It implements `Deref<Target=T>`.
    ///
    /// All associated functions are only usefull for the implementation of
    /// the `dynamic` proc macro attribute
    pub struct ConstStatic<T>(UnsafeCell<Static<T>>);

  
  
  use core::sync::atomic::{AtomicI32, Ordering};
  
  static CUR_INIT_PRIO: AtomicI32 = AtomicI32::new(i32::MIN);
  
  static CUR_DROP_PRIO: AtomicI32 = AtomicI32::new(i32::MIN);
  
  #[doc(hidden)]
  #[inline]
  pub fn __set_init_prio(v: i32) {
      CUR_INIT_PRIO.store(v, Ordering::Relaxed);
  }

  impl<T> Static<T> {
      #[inline]
      pub const fn uninit(info: StaticInfo) -> Self {
              Self(StaticBase { k: () }, info, AtomicI32::new(0))
      }
      #[inline]
      pub const fn from(v: T, info: StaticInfo) -> Self {
              Static(
                  StaticBase {
                      v: ManuallyDrop::new(v),
                  },
                  info,
                  AtomicI32::new(1),
              )
      }
  
      #[inline]
      pub unsafe fn set_to(this: &mut Self, v: T) {
              this.0.v = ManuallyDrop::new(v);
              this.2.store(1, Ordering::Relaxed);
      }
  
      #[inline]
      pub unsafe fn drop(this: &mut Self) {
              if let DropMode::Dynamic(prio) = &this.1.drop_mode {
                  CUR_DROP_PRIO.store(*prio as i32, Ordering::Relaxed);
                  ManuallyDrop::drop(&mut this.0.v);
                  CUR_DROP_PRIO.store(i32::MIN, Ordering::Relaxed);
              } else {
                  ManuallyDrop::drop(&mut this.0.v);
              };
              this.2.store(2, Ordering::Relaxed);
      }
  }
  
  #[inline]
  fn check_access(info: &StaticInfo, status: i32) {
      if status == 0 {
          core::panic!(
              "Attempt to access variable {:#?} before it is initialized during initialization \
               priority {}. Tip: increase init priority of this static to a value larger than \
               {prio} (attribute syntax: `#[dynamic(init=<prio>)]`)",
              info,
              prio = CUR_INIT_PRIO.load(Ordering::Relaxed)
          )
      }
      if status == 2 {
          core::panic!(
              "Attempt to access variable {:#?} after it was destroyed during destruction priority \
               {prio}. Tip increase drop priority of this static to a value larger than {prio} \
               (attribute syntax: `#[dynamic(drop=<prio>)]`)",
              info,
              prio = CUR_DROP_PRIO.load(Ordering::Relaxed)
          )
      }
      let init_prio = CUR_INIT_PRIO.load(Ordering::Relaxed);
      let drop_prio = CUR_DROP_PRIO.load(Ordering::Relaxed);
  
      if let DropMode::Dynamic(prio) = &info.drop_mode {
          if drop_prio == *prio as i32 {
              core::panic!(
                  "This access to variable {:#?} is not sequenced before to its drop. Tip increase drop \
                   priority of this static to a value larger than {prio} (attribute syntax: \
                   `#[dynamic(drop=<prio>)]`)",
                  info,
                  prio = drop_prio
              )
          } else if drop_prio > *prio as i32 {
          core::panic!(
              "Unexpected initialization order while accessing {:#?} from drop \
               priority {}. This is a bug of `static_init` library, please report \"
             the issue inside `static_init` repository.",
              info,
              drop_prio
          )
          }
      } 
  
      if let InitMode::Dynamic(prio) = &info.init_mode {
          if init_prio == *prio as i32 {
              core::panic!(
                  "This access to variable {:#?} is not sequenced after construction of this static. \
                   Tip increase init priority of this static to a value larger than {prio} (attribute \
                   syntax: `#[dynamic(init=<prio>)]`)",
                  info,
                  prio = init_prio
              )
          } else if init_prio > *prio as i32 {
          core::panic!(
              "Unexpected initialization order while accessing {:#?} from init priority {}\
               . This is a bug of `static_init` library, please report \"
             the issue inside `static_init` repository.",
              info,
              init_prio,
          )
          }
      } 
  }
  
  impl<T> Deref for Static<T> {
      type Target = T;
      #[inline(always)]
      fn deref(&self) -> &T {
          check_access(&self.1, self.2.load(Ordering::Relaxed));
          unsafe { &*self.0.v }
      }
  }
  impl<T> DerefMut for Static<T> {
      #[inline(always)]
      fn deref_mut(&mut self) -> &mut T {
          check_access(&self.1, self.2.load(Ordering::Relaxed));
          unsafe { &mut *self.0.v }
      }
  }

    impl<T> ConstStatic<T> {
        #[inline]
        pub const fn uninit(info: StaticInfo) -> Self {
            Self(UnsafeCell::new(Static::uninit(info)))
        }
        #[inline]
        pub const fn from(v: T, info: StaticInfo) -> Self {
            Self(UnsafeCell::new(Static::from(v, info)))
        }
        #[inline]
        pub unsafe fn set_to(this: &Self, v: T) {
            Static::set_to(&mut (*this.0.get()), v)
        }
        #[inline]
        pub unsafe fn drop(this: &Self) {
            Static::drop(&mut *this.0.get());
        }
    }
    
    unsafe impl<T: Send> Send for ConstStatic<T> {}
    unsafe impl<T: Sync> Sync for ConstStatic<T> {}
    
    impl<T> Deref for ConstStatic<T> {
        type Target = T;
        #[inline(always)]
        fn deref(&self) -> &T {
            unsafe { &**self.0.get() }
        }
    }
}

#[cfg(not(debug_mode))]
mod static_impl {
  use core::mem::ManuallyDrop;
  use core::ops::{Deref,DerefMut};
  use super::StaticBase;
    use core::cell::UnsafeCell;
  /// The actual type of mutable *dynamic statics*.
  ///
  /// It implements `Deref<Target=T>` and `DerefMut`.
  ///
  /// All associated functions are only usefull for the implementation of
  /// the `dynamic` proc macro attribute
  pub struct Static<T>(
      StaticBase<T>,
  );

  /// The actual type of non mutable *dynamic statics*.
  ///
  /// It implements `Deref<Target=T>`.
  ///
  /// All associated functions are only usefull for the implementation of
  /// the `dynamic` proc macro attribute
  pub struct ConstStatic<T>(UnsafeCell<Static<T>>);

  
  
  #[doc(hidden)]
  #[inline(always)]
  pub fn __set_init_prio(_: i32) {}
  
  //As a trait in order to avoid noise;
  impl<T> Static<T> {
      #[inline]
      pub const fn uninit() -> Self {
          Self(StaticBase { k: () })
      }
      #[inline]
      pub const fn from(v: T) -> Self {
         Static(StaticBase {
             v: ManuallyDrop::new(v),
         })
      }
  
      #[inline]
      pub unsafe fn set_to(this: &mut Self, v: T) {
          this.0.v = ManuallyDrop::new(v);
      }
  
      #[inline]
      pub unsafe fn drop(this: &mut Self) {
              ManuallyDrop::drop(&mut this.0.v);
      }
  }
  
  impl<T> Deref for Static<T> {
      type Target = T;
      #[inline(always)]
      fn deref(&self) -> &T {
          unsafe { &*self.0.v }
      }
  }
  impl<T> DerefMut for Static<T> {
      #[inline(always)]
      fn deref_mut(&mut self) -> &mut T {
          unsafe { &mut *self.0.v }
      }
  }

    impl<T> ConstStatic<T> {
        #[inline]
        pub const fn uninit() -> Self {
            Self(UnsafeCell::new(Static::uninit()))
        }
        #[inline]
        pub const fn from(v: T) -> Self {
            Self(UnsafeCell::new(Static::from(v)))
        }
        #[inline]
        pub unsafe fn set_to(this: &Self, v: T) {
            Static::set_to(&mut (*this.0.get()), v)
        }
        #[inline]
        pub unsafe fn drop(this: &Self) {
            Static::drop(&mut *this.0.get());
        }
    }
    
    unsafe impl<T: Send> Send for ConstStatic<T> {}
    unsafe impl<T: Sync> Sync for ConstStatic<T> {}
    
    impl<T> Deref for ConstStatic<T> {
        type Target = T;
        #[inline(always)]
        fn deref(&self) -> &T {
            unsafe { &**self.0.get() }
        }
    }
}
