//! **This crate has been deprecated.
//! Rust 1.61 stabilized `retain_mut` for `Vec` and `VecDeque`,
//! so you can use them directly.
//! This crate is no longer maintained.**
//!
//! This crate provides trait `RetainMut` which
//! provides `retain_mut` method for `Vec` and `VecDeque`.
//!
//! `retain_mut` is basically the same as `retain` except that
//! it gives mutable reference of items to the predicate function.
//!
//! Since there is no reason `retain` couldn't have been designed this way,
//! this crate basically just copies the code from std with minor changes
//! to hand out mutable reference.
//! The code these impls are based on can be found in code comments of this crate.
//!
//! This was probably a historical mistake in Rust library,
//! that `retain` should do this at the very beginning.
//! See [rust-lang/rust#25477](https://github.com/rust-lang/rust/issues/25477).
//!
//! From Rust 1.58, an unstable `retain_mut` method has been added to the std, see
//! [rust-lang/rust#90829](https://github.com/rust-lang/rust/issues/90829).
//! Once it gets stabilized, you can simply remove this crate.
//!
//! ## Examples
//!
//! ### `Vec`
//!
//! ```
//! # use retain_mut::RetainMut;
//! let mut vec = vec![1, 2, 3, 4];
//! vec.retain_mut(|x| { *x *= 3; *x % 2 == 0 });
//! assert_eq!(vec, [6, 12]);
//! ```
//!
//! ### `VecDeque`
//!
//! ```
//! # use retain_mut::RetainMut;
//! # use std::collections::VecDeque;
//! let mut deque = VecDeque::from(vec![1, 2, 3, 4]);
//! deque.retain_mut(|x| { *x *= 3; *x % 2 == 0 });
//! assert_eq!(deque, [6, 12]);
//! ```

#![no_std]

extern crate alloc;

use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;
use core::ptr;

/// Trait that provides `retain_mut` method.
#[deprecated = "Rust 1.61 has included retain_mut directly"]
pub trait RetainMut<T> {
    /// Retains only the elements specified by the predicate, passing a mutable reference to it.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool;
}

impl<T> RetainMut<T> for Vec<T> {
    // The implementation is based on
    // https://github.com/rust-lang/rust/blob/03c8ffaacb040a8753ef8e1accea701bc9f5be85/library/alloc/src/vec/mod.rs#L1478-L1569
    fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let original_len = self.len();
        // Avoid double drop if the drop guard is not executed,
        // since we may make some holes during the process.
        unsafe { self.set_len(0) };

        // Vec: [Kept, Kept, Hole, Hole, Hole, Hole, Unchecked, Unchecked]
        //      |<-              processed len   ->| ^- next to check
        //                  |<-  deleted cnt     ->|
        //      |<-              original_len                          ->|
        // Kept: Elements which predicate returns true on.
        // Hole: Moved or dropped element slot.
        // Unchecked: Unchecked valid elements.
        //
        // This drop guard will be invoked when predicate or `drop` of element panicked.
        // It shifts unchecked elements to cover holes and `set_len` to the correct length.
        // In cases when predicate and `drop` never panick, it will be optimized out.
        struct BackshiftOnDrop<'a, T> {
            v: &'a mut Vec<T>,
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        impl<T> Drop for BackshiftOnDrop<'_, T> {
            fn drop(&mut self) {
                if self.deleted_cnt > 0 {
                    // SAFETY: Trailing unchecked items must be valid since we never touch them.
                    unsafe {
                        ptr::copy(
                            self.v.as_ptr().add(self.processed_len),
                            self.v
                                .as_mut_ptr()
                                .add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len,
                        );
                    }
                }
                // SAFETY: After filling holes, all items are in contiguous memory.
                unsafe {
                    self.v.set_len(self.original_len - self.deleted_cnt);
                }
            }
        }

        let mut g = BackshiftOnDrop {
            v: self,
            processed_len: 0,
            deleted_cnt: 0,
            original_len,
        };

        fn process_loop<F, T, const DELETED: bool>(
            original_len: usize,
            f: &mut F,
            g: &mut BackshiftOnDrop<'_, T>,
        ) where
            F: FnMut(&mut T) -> bool,
        {
            while g.processed_len != original_len {
                // SAFETY: Unchecked element must be valid.
                let cur = unsafe { &mut *g.v.as_mut_ptr().add(g.processed_len) };
                if !f(cur) {
                    // Advance early to avoid double drop if `drop_in_place` panicked.
                    g.processed_len += 1;
                    g.deleted_cnt += 1;
                    // SAFETY: We never touch this element again after dropped.
                    unsafe { ptr::drop_in_place(cur) };
                    // We already advanced the counter.
                    if DELETED {
                        continue;
                    } else {
                        break;
                    }
                }
                if DELETED {
                    // SAFETY: `deleted_cnt` > 0, so the hole slot must not overlap with current element.
                    // We use copy for move, and never touch this element again.
                    unsafe {
                        let hole_slot = g.v.as_mut_ptr().add(g.processed_len - g.deleted_cnt);
                        ptr::copy_nonoverlapping(cur, hole_slot, 1);
                    }
                }
                g.processed_len += 1;
            }
        }

        // Stage 1: Nothing was deleted.
        process_loop::<F, T, false>(original_len, &mut f, &mut g);

        // Stage 2: Some elements were deleted.
        process_loop::<F, T, true>(original_len, &mut f, &mut g);

        // All item are processed. This can be optimized to `set_len` by LLVM.
        drop(g);
    }
}

impl<T> RetainMut<T> for VecDeque<T> {
    // The implementation is based on
    // https://github.com/rust-lang/rust/blob/3e21768a0a3fc84befd1cbe825ae6849e9941b73/library/alloc/src/collections/vec_deque/mod.rs#L2148-L2180
    fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let len = self.len();
        let mut idx = 0;
        let mut cur = 0;

        // Stage 1: All values are retained.
        while cur < len {
            if !f(&mut self[cur]) {
                cur += 1;
                break;
            }
            cur += 1;
            idx += 1;
        }
        // Stage 2: Swap retained value into current idx.
        while cur < len {
            if !f(&mut self[cur]) {
                cur += 1;
                continue;
            }

            self.swap(idx, cur);
            cur += 1;
            idx += 1;
        }
        // Stage 3: Trancate all values after idx.
        if cur != idx {
            self.truncate(idx);
        }
    }
}
