[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![LICENSE](https://img.shields.io/badge/license-apache-blue.svg)](LICENSE-APACHE)
[![Documentation](https://docs.rs/static_init/badge.svg)](https://docs.rs/static_init)
[![Crates.io Version](https://img.shields.io/crates/v/static_init.svg)](https://crates.io/crates/static_init)


Non const static initialization, and program constructor/destructor code.

# Lesser Lazy Statics

This crate provides *lazy statics* on all plateforms.

On unixes and windows *lesser lazy statics* are *lazy* during program startup phase
(before `main` is called). Once main is called, those statics are all guaranteed to be
initialized and any access to them is as fast as any access to regular const initialized
statics. Benches sho that usual lazy statics, as those provided by `std::lazy::*` or from
[lazy_static][1] crate, suffer from a 2ns access penalty.

*Lesser lazy statics* can optionaly be dropped at program destruction
(after main exit but before the program stops).

*Lesser lazy statics* require the standard library and are enabled by default
crate features `lazy` and `atexit`.
```rust
use static_init::{dynamic};

#[dynamic] //equivalent to #[dynamic(lazy)]
static L1: Vec<i32> = unsafe{L0.clone()};

#[dynamic(drop)] //equivalent to #[dynamic(lazy,drop)]
static L0: Vec<i32> = vec![1,2,3];

#[dynamic(drop)]
static mut L2: Vec<i32> = L1.clone();
```
As can be seen above accesses to *lazy static* that are dropped must be within unsafe
blocks. The reason is that it is possible at program destruction to access already dropped
lazy statics.

# Dynamic statics: statics initialized at program startup

On plateforms that support it (unixes, mac, windows), this crate provides *dynamic statics*: statics that are
initialized at program startup. This feature is `no_std`.

```rust
use static_init::{dynamic};

#[dynamic(0)]
//equivalent to #[dynamic(init=0)]
static D1: Vec<i32> = vec![1,2,3];

assert_eq!(unsafe{D1[0]}, 1);
```
As can be seen above, even if D1 is not mutable, access to it must be performed in unsafe
blocks. The reason is that during startup phase, accesses to *dynamic statics* may cause
*undefined behavior*: *dynamic statics* may be in a zero initialized state.

To prevent such hazardeous accesses, on unixes and window plateforms, a priority can be
specified. Dynamic static initializations with higher priority are sequenced before dynamic
static initializations with lower priority. Dynamic static initializations with the same
priority are underterminately sequenced.

```rust
use static_init::{dynamic};

// D2 initialization is sequenced before D1 initialization
#[dynamic(0)]
static mut D1: Vec<i32> = unsafe{D2.clone()};

#[dynamic(10)]
static D2: Vec<i32> = vec![1,2,3];
```

*Dynamic statics* can be dropped at program destruction phase: they are dropped after main
exit:

```rust
use static_init::{dynamic};

// D2 initialization is sequenced before D1 initialization
// D1 drop is sequenced before D2 drop.
#[dynamic(init=0,drop=0)]
static mut D1: Vec<i32> = unsafe {D2.clone()};

#[dynamic(init=10,drop=10)]
static D2: Vec<i32> = vec![1,2,3];
```
The priority act on drop in reverse order. *Dynamic statics* drops with a lower priority are
sequenced before *dynamic statics* drops with higher priority.

Finally, if the feature `atexit` is enabled, *dynamic statics* drop can be registered with
`libc::atexit`. *lazy dynamic statics* and *dynamic statics* with `drop_reverse` attribute
argument are destroyed in the reverse order of their construction. Functions registered with
`atexit` are executed before program destructors and drop of *dynamic statics* that use the
`drop` attribute argument. Drop is registered with at `atexit` if no priority if given to the
`drop` attribute argument.

```rust
use static_init::{dynamic};

//D1 is dropped before D2 because
//it is initialized before D2
#[dynamic(lazy,drop)]
static D1: Vec<i32> = vec![0,1,2];

#[dynamic(10,drop)]
static D2: Vec<i32> = unsafe{D1.clone()};

//D3 is initilized after D1 and D2 initializations
//and it is dropped after D1 and D2 drops
#[dynamic(5,drop)]
static D3: Vec<i32> = unsafe{D1.clone()};
```

# Constructor and Destructor

On plateforms that support it (unixes, mac, windows), this crate provides a way to declare
*constructors*: a function called before main is called. This feature is `no_std`.

```rust
use static_init::{constructor};

//called before main
#[constructor] //equivalent to #[constructor(0)]
extern "C" fn some_init() {}
```

Constructors also support priorities. Sequencement rules applies also between constructor calls and
between *dynamic statics* initialization and *constructor* calls.

*destructors* are called at program destruction. They also support priorities.

```rust
use static_init::{constructor, destructor};

//called before some_init
#[constructor(10)]
extern "C" fn pre_init() {}

//called before main
#[constructor]
extern "C" fn some_init() {}

//called after main
#[destructor]
extern "C" fn first_destructor() {}

//called after first_destructor
#[destructor(10)]
extern "C" fn last_destructor() {}
```

# Thread Local Support

Variable declared with `#[dynamic(lazy)]` can also be declared `#[thread_local]`. These
variable will behave as regular *lazy statics*.

```rust
#[thread_local]
#[dynamic(lazy)]
static mut X: Vec<i32> = vec![1,2,3];
```

These variables can also be droped on thread exit.

```rust
#[thread_local]
#[dynamic(lazy,drop)]
static X: Vec<i32> = vec![1,2,3];

assert!(unsafe{X[1] == 2});
```

Accessing a thread local *lazy statics* that should drop during the phase where thread_locals are
droped may cause *undefined behavior*. For this reason any access to a thread local lazy static
that is dropped will require an unsafe block, even if the static is const.

# Debuging initialization order

If the feature `debug_order` is enabled, attempts to access `dynamic statics` that are
uninitialized or whose initialization is undeterminately sequenced with the access will cause
a panic with a message specifying which statics was tentatively accessed and how to change this
*dynamic static* priority to fix this issue.

Run `cargo test` in this crate directory to see message examples.

All implementations of lazy statics may suffer from circular initialization dependencies. Those
circular dependencies will cause either a dead lock or an infinite loop. If the feature `debug_order` is
enabled, atemp are made to detect those circular dependencies. In most case they will be detected.



# Comparisons with other crates

## Comparison of *Lesser lazy statics* with [lazy_static][1] or `std::lazy::Lazy`.
 - lazy_static only provides const statics;
 - there are no cyclic initialization detection;
 - Each access to lazy_static statics costs 2ns;
 - syntax is more verbose.

## *dynamic statics* with [ctor][2]
 - ctor only provides const statics;
 - ctor does not provide priorities;
 - ctor unsafety is unsound;
 - ctor does not support mutable statics;
 - ctor does not provide a way to detect access to uninitialized data.

[1]: https://crates.io/crates/lazy_static
[2]: https://crates.io/crates/ctor
