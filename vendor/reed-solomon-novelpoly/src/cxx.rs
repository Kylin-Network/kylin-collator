#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

mod cxx {
	include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use crate::Shard;

fn setup() {
	use std::sync::Once;

	static SETUP: Once = Once::new();

	SETUP.call_once(|| unsafe {
		cxx::setup();
	});
}

pub fn encode<S: Shard>(_bytes: &[u8]) -> Vec<S> {
	setup();
	unimplemented!("encode for C for usage in rs bench is not implemented")
}

pub fn reconstruct<S: Shard>(_received_shards: Vec<Option<S>>) -> Option<Vec<u8>> {
	setup();
	unimplemented!("reconstruction for C for usage in rs bench is not implemented")
}

#[cfg(test)]
mod tests {
	use rand::prelude::SmallRng;
	use rand::Rng;

	use crate::novel_poly_basis::{fft_in_novel_poly_basis, GFSymbol};

	use super::*;

	#[test]
	fn konst() {
		assert_eq!(cxx::FIELD_SIZE as usize, crate::f2e16::FIELD_SIZE);
		assert_eq!(cxx::FIELD_BITS as usize, crate::f2e16::FIELD_BITS);
		assert_eq!(cxx::MODULO as u16, crate::f2e16::MODULO);
		itertools::assert_equal(unsafe { &cxx::Base }.iter(), crate::f2e16::BASE.iter());
	}

	#[test]
	fn c_tests() {
		assert_eq!(unsafe { cxx::test_flt_roundtrip() }, 0);
	}

	#[test]
	fn flt_forward() {
		let mut rng = rand::thread_rng();

		let data: [GFSymbol; 32] = rng.gen();
		let dl = data.len();
		let cxx = {
			let mut data = data.clone();

			unsafe {
				cxx::FLT(data.as_mut_ptr(), dl as i32, 0_i32);
			}
			data
		};
		let rust = {
			let mut data = data.clone();
			novel_poly_basis::fft_in_novel_poly_basis(&mut data[..], dl, 0);
			data
		};
		assert_eq!(cxx, rust);
	}

	#[test]
	fn flt_backward() {
		let mut rng = rand::thread_rng();

		let data: [GFSymbol; 32] = rng.gen();
		let dl = data.len();
		let cxx = {
			let mut data = data.clone();

			unsafe {
				cxx::IFLT(data.as_mut_ptr(), dl as i32, 0);
			}
			data
		};
		let rust = {
			let mut data = data.clone();
			novel_poly_basis::inverse_fft_in_novel_poly_basis(&mut data[..], dl, 0);
			data
		};
		assert_eq!(cxx, rust);
	}
}
