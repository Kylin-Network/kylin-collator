// Copyright 2017 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Means of mixing a series of hashes to create a single secure hash.
//!
//! Described in http://www.cs.huji.ac.il/~nati/PAPERS/coll_coin_fl.pdf

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std::ops::{BitAnd, BitOr};

#[cfg(not(feature = "std"))]
use core::ops::{BitAnd, BitOr};

pub const MAX_DEPTH: usize = 17;

fn sub_mix<T>(seeds: &[T]) -> T where
	T: BitAnd<Output = T> + BitOr<Output = T> + Copy
{
	(seeds[0] & seeds[1]) | (seeds[1] & seeds[2]) | (seeds[0] & seeds[2])
}

/// Mix a slice.
pub fn triplet_mix<T>(seeds: &[T]) -> Result<T, ()> where
	T: BitAnd<Output = T> + BitOr<Output = T>,
	T: Default + Copy
{
	Ok(seeds.iter().cloned().triplet_mix())
}

/// The mixed trait for mixing a sequence.
pub trait TripletMix {
	/// The items in the sequence and simultaneously the return of the mixing.
	type Item;
	/// The output of the mixing algorithm on the sequence. Items in the sequence beyond
	/// the largest power of three that fits within the the sequence up until `3 ** MAX_DEPTH`
	/// are ignored.
	fn triplet_mix(self) -> Self::Item;
}

impl<I, T> TripletMix for I where
	I: Iterator<Item = T>,
	T: BitAnd<Output = T> + BitOr<Output = T> + Default + Copy
{
	type Item = T;
	fn triplet_mix(self) -> Self::Item {
		let mut accum = [[T::default(); 3]; MAX_DEPTH];
		let mut result = T::default();
		for (i, seed) in self.enumerate() {
			accum[0][i % 3] = seed;
			let mut index_at_depth = i;
			for depth in 0..MAX_DEPTH {
				if index_at_depth % 3 != 2 {
					break;
				}
				index_at_depth /= 3;
				result = sub_mix(&accum[depth]);

				// end of the threesome at depth.
				if depth == MAX_DEPTH - 1 {
					// end of our stack - bail with result.
					break;
				} else {
					// save in the stack for parent computation
					accum[depth + 1][index_at_depth % 3] = result;
				}
			}
		}
		result
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn sub_mix_works() {
		assert_eq!(sub_mix(&[0, 0, 0][..]), 0);
		assert_eq!(sub_mix(&[0, 0, 1][..]), 0);
		assert_eq!(sub_mix(&[0, 1, 0][..]), 0);
		assert_eq!(sub_mix(&[0, 1, 1][..]), 1);
		assert_eq!(sub_mix(&[1, 0, 0][..]), 0);
		assert_eq!(sub_mix(&[1, 0, 1][..]), 1);
		assert_eq!(sub_mix(&[1, 1, 0][..]), 1);
		assert_eq!(sub_mix(&[1, 1, 1][..]), 1);

		assert_eq!(sub_mix(&[0, 0, 0][..]), 0);
		assert_eq!(sub_mix(&[0, 0, 2][..]), 0);
		assert_eq!(sub_mix(&[0, 2, 0][..]), 0);
		assert_eq!(sub_mix(&[0, 2, 2][..]), 2);
		assert_eq!(sub_mix(&[2, 0, 0][..]), 0);
		assert_eq!(sub_mix(&[2, 0, 2][..]), 2);
		assert_eq!(sub_mix(&[2, 2, 0][..]), 2);
		assert_eq!(sub_mix(&[2, 2, 2][..]), 2);
	}

	#[test]
	fn triplet_mix_works_on_first_level() {
		assert_eq!(triplet_mix(&[0, 0, 0][..]).unwrap(), 0);
		assert_eq!(triplet_mix(&[0, 0, 1][..]).unwrap(), 0);
		assert_eq!(triplet_mix(&[0, 1, 0][..]).unwrap(), 0);
		assert_eq!(triplet_mix(&[0, 1, 1][..]).unwrap(), 1);
		assert_eq!(triplet_mix(&[1, 0, 0][..]).unwrap(), 0);
		assert_eq!(triplet_mix(&[1, 0, 1][..]).unwrap(), 1);
		assert_eq!(triplet_mix(&[1, 1, 0][..]).unwrap(), 1);
		assert_eq!(triplet_mix(&[1, 1, 1][..]).unwrap(), 1);

		assert_eq!(triplet_mix(&[0, 0, 0][..]).unwrap(), 0);
		assert_eq!(triplet_mix(&[0, 0, 2][..]).unwrap(), 0);
		assert_eq!(triplet_mix(&[0, 2, 0][..]).unwrap(), 0);
		assert_eq!(triplet_mix(&[0, 2, 2][..]).unwrap(), 2);
		assert_eq!(triplet_mix(&[2, 0, 0][..]).unwrap(), 0);
		assert_eq!(triplet_mix(&[2, 0, 2][..]).unwrap(), 2);
		assert_eq!(triplet_mix(&[2, 2, 0][..]).unwrap(), 2);
		assert_eq!(triplet_mix(&[2, 2, 2][..]).unwrap(), 2);
	}

	#[test]
	fn triplet_mix_works_on_second_level() {
		assert_eq!(triplet_mix(&[0, 0, 0, 0, 0, 1, 0, 1, 0][..]).unwrap(), 0);
		assert_eq!(triplet_mix(&[0, 1, 1, 1, 0, 0, 1, 0, 1][..]).unwrap(), 1);
		assert_eq!(triplet_mix(&[1, 1, 0, 1, 1, 1, 0, 0, 0][..]).unwrap(), 1);
	}

	#[test]
	fn triplet_mix_works_on_third_level() {
		assert_eq!(triplet_mix(&[0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0][..]).unwrap(), 1);
	}
}
