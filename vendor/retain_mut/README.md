# RetainMut

<!-- cargo-sync-readme start -->

**This crate has been deprecated.
Rust 1.61 stabilized `retain_mut` for `Vec` and `VecDeque`,
so you can use them directly.
This crate is no longer maintained.**

This crate provides trait `RetainMut` which
provides `retain_mut` method for `Vec` and `VecDeque`.

`retain_mut` is basically the same as `retain` except that
it gives mutable reference of items to the predicate function.

Since there is no reason `retain` couldn't have been designed this way,
this crate basically just copies the code from std with minor changes
to hand out mutable reference.
The code these impls are based on can be found in code comments of this crate.

This was probably a historical mistake in Rust library,
that `retain` should do this at the very beginning.
See [rust-lang/rust#25477](https://github.com/rust-lang/rust/issues/25477).

From Rust 1.58, an unstable `retain_mut` method has been added to the std, see
[rust-lang/rust#90829](https://github.com/rust-lang/rust/issues/90829).
Once it gets stabilized, you can simply remove this crate.

## Examples

### `Vec`

```rust
let mut vec = vec![1, 2, 3, 4];
vec.retain_mut(|x| { *x *= 3; *x % 2 == 0 });
assert_eq!(vec, [6, 12]);
```

### `VecDeque`

```rust
let mut deque = VecDeque::from(vec![1, 2, 3, 4]);
deque.retain_mut(|x| { *x *= 3; *x % 2 == 0 });
assert_eq!(deque, [6, 12]);
```

<!-- cargo-sync-readme end -->
