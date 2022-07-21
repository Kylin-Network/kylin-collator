<div class="title-block" style="text-align: center;" align="center">

# `wyz`

## myrrlyn’s wyzyrdly library <!-- omit in toc -->

[![Crate][crate_img]][crate]
[![Documentation][docs_img]][docs]
[![License][license_img]][license_file]

[![Crate Downloads][downloads_img]][crate]
[![Crate Size][loc_img]][loc]

</div>

I have developed a collection of utility and convenience Rust modules that are
useful to me, and may be useful to you also.

This crate is a collection of largely-independent small modules. I do not
currently offer features to disable modules independently of each other, but
their compilation cost is small enough to essentially not matter.

## Modules <!-- omit in toc -->

1. [`bidi`](#bidi)
1. [`comu`](#comu)
1. [`exit`](#exit)
1. [`fmt`](#fmt)
1. [`range`](#range)
1. [`wm`](#wm)

## `bidi`

This provides an extension trait for `DoubleEndedIterator` with a method,
`.bidi(cond: bool)`, that sets whether the iterator operates in forward or
reverse by the runtime condition. When the condition is `true`, forward
iteration (with `.next()`, `.nth()`) forwards to the equivalent reverse
methods (`.next_back()`, `.nth_back()`) and vice-versa; when the condition is
`false`, iteration behaves normally.

This only checks the condition upon initial creation; it is otherwise
branchless.

## `comu`

This provides a generalization system for mutability, pointers, and references.
It lifts the `const` and `mut` *name permissions* into the type system, with
the `Const`, `Mut`, and `Frozen<M>` types, and allows containers to generalize
over mutability permissions with the `Mutability` trait.

Additionally, it provides the `Address` and `Reference` types, which are
equivalent to pointers and references. `Address<Const, T>` is `*const T`,
`Address<Mut, T>` is `*mut T`, `Reference<Const, T>` is `&T`, and
`Reference<Mut, T>` is `&mut T`.

These do not necessarily have a complete port of the pointer and reference
fundamental APIs, as this module is primarily written as a minimum product
necessary for `bitvec`’s use, rather than a goal in its own right. PRs welcome!

## `exit`

This is a macro that calls `std::process::exit`. It can return a status code,
and also print a message to `stderr`.

```rust
use wyz::exit::exit;

exit!();
exit!(2);
exit!(3, "This is a {} message", "failure");
```

The default call is `std::process::exit(1)`; a call may provide its own exit
code and, in addition, a set of arguments to pass directly to `eprintln!`. The
error message is not guaranteed to be emitted, as `stderr` may be closed at time
of `exit!`.

## `fmt`

Rust uses the `Debug` trait for automatic printing events in several parts of
the standard library. This module provides wrapper types which forward their
`Debug` implementation to a specified other formatting trait. It also implements
extension methods on all types that have format trait implementations to wrap
them in the corresponding shim type.

```rust
use wyz::fmt::FmtForward as _;

let val = 6;
let addr = &val as *const i32;
println!("{:?}", addr.fmt_pointer());
```

This snippet uses the `Debug` format template, but will print the `Pointer`
implementation of `*const i32`.

This is useful for fitting your values into an error-handling framework that
only uses `Debug`, such as the `fn main() -> Result` program layout.

In addition to forwarding each of the scalar traits, this also provides a
`.fmt_list()` that formats any type `T where &T: IntoIterator` as a list. The
list-formatting adapter itself implements all of the scalar formatting traits,
and can also be wrapped in any of the forwarding guards so that it can be sent
to a `Debug` sink:

```rust
use wyz::fmt::FmtForward as _;

let seq = 0 .. 4;
assert_eq!(
  format!("{:02b}", seq.fmt_list()),
  "[00, 01, 10, 11]",
);
assert_eq!(
  format!(
    "{:?}",
    seq
      .map(|x| (x + 1) * 10)
      .fmt_list()
      .fmt_lower_hex(),
  ),
  "[a, 14, 1e, 28]",
);
```

## `range`

This provides an extension trait, `RangeExt`, on `RangeBounds`. It is currently
only used with `R: RangeBounds<usize>`, again because it is an MVP for bitvec’s
use rather than a project in its own right. It normalizes arbitrary ranges into
the `Range` concrete type. PRs welcome!

## `wm`

This is an experimental module whose sole purpose is to implement an off-thread
destructor. It works by wrapping `Send` types so that when they go out of scope,
the value is sent to a background thread whose sole job is to take objects off
the queue and run their destructor. The queue is infinite and non-blocking, so
your primary workers can be guaranteed that transmission will not stall them
unduly.

[crate]: https://crates.io/crates/wyz "Crate Link"
[crate_img]: https://img.shields.io/crates/v/wyz.svg?logo=rust "Crate Page"
[docs]: https://docs.rs/wyz "Documentation"
[docs_img]: https://docs.rs/wyz/badge.svg "Documentation Display"
[downloads_img]: https://img.shields.io/crates/dv/wyz.svg?logo=rust "Crate Downloads"
[license_file]: https://github.com/myrrlyn/wyz/blob/master/LICENSE.txt "License File"
[license_img]: https://img.shields.io/crates/l/wyz.svg "License Display"
[loc]: https://github.com/myrrlyn/wyz "Repository"
[loc_img]: https://tokei.rs/b1/github/myrrlyn/wyz?category=code "Repository Size"

<style type="text/css">
.title-block {
  text-align: center;
}
</style>
