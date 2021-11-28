//! Autogenerated weights for kylin_oracle
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2021-11-28, STEPS: `100`, REPEAT: 200, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("pichiu-chachacha"), DB CACHE: 128

// Executed Command:
// target/release/kylin-collator
// benchmark
// --chain=pichiu-chachacha
// --steps=100
// --repeat=200
// --pallet=kylin-oracle
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=example_weights.rs
// --template=./scripts/frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for kylin_oracle.
pub trait WeightInfo {	fn sudo_remove_feed_account(a: u32, ) -> Weight;	fn submit_price_feed(a: u32, ) -> Weight;	fn query_data(a: u32, ) -> Weight;	fn write_data_onchain(a: u32, ) -> Weight;	fn submit_data_signed(a: u32, ) -> Weight;	fn submit_data_unsigned(a: u32, ) -> Weight;	fn submit_data_via_api(a: u32, ) -> Weight;	fn clear_api_queue_unsigned(a: u32, ) -> Weight;}

/// Weights for kylin_oracle using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {	fn sudo_remove_feed_account(a: u32, ) -> Weight {
		(39_825_000 as Weight)			// Standard Error: 0
			.saturating_add((1_000 as Weight).saturating_mul(a as Weight))			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn submit_price_feed(_a: u32, ) -> Weight {
		(65_555_000 as Weight)			.saturating_add(T::DbWeight::get().reads(3 as Weight))			.saturating_add(T::DbWeight::get().writes(3 as Weight))	}	fn query_data(a: u32, ) -> Weight {
		(120_431_000 as Weight)			// Standard Error: 0
			.saturating_add((4_000 as Weight).saturating_mul(a as Weight))			.saturating_add(T::DbWeight::get().reads(4 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}	fn write_data_onchain(_a: u32, ) -> Weight {
		(63_382_000 as Weight)			.saturating_add(T::DbWeight::get().reads(3 as Weight))			.saturating_add(T::DbWeight::get().writes(3 as Weight))	}	fn submit_data_signed(_a: u32, ) -> Weight {
		(20_506_000 as Weight)			.saturating_add(T::DbWeight::get().reads(2 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn submit_data_unsigned(_a: u32, ) -> Weight {
		(25_216_000 as Weight)			.saturating_add(T::DbWeight::get().reads(2 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn submit_data_via_api(a: u32, ) -> Weight {
		(65_295_000 as Weight)			// Standard Error: 0
			.saturating_add((1_000 as Weight).saturating_mul(a as Weight))			.saturating_add(T::DbWeight::get().reads(3 as Weight))			.saturating_add(T::DbWeight::get().writes(3 as Weight))	}	fn clear_api_queue_unsigned(_a: u32, ) -> Weight {
		(6_141_000 as Weight)			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}}

// For backwards compatibility and tests
impl WeightInfo for () {	fn sudo_remove_feed_account(a: u32, ) -> Weight {
		(39_825_000 as Weight)			// Standard Error: 0
			.saturating_add((1_000 as Weight).saturating_mul(a as Weight))			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn submit_price_feed(_a: u32, ) -> Weight {
		(65_555_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(3 as Weight))	}	fn query_data(a: u32, ) -> Weight {
		(120_431_000 as Weight)			// Standard Error: 0
			.saturating_add((4_000 as Weight).saturating_mul(a as Weight))			.saturating_add(RocksDbWeight::get().reads(4 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}	fn write_data_onchain(_a: u32, ) -> Weight {
		(63_382_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(3 as Weight))	}	fn submit_data_signed(_a: u32, ) -> Weight {
		(20_506_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn submit_data_unsigned(_a: u32, ) -> Weight {
		(25_216_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn submit_data_via_api(a: u32, ) -> Weight {
		(65_295_000 as Weight)			// Standard Error: 0
			.saturating_add((1_000 as Weight).saturating_mul(a as Weight))			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(3 as Weight))	}	fn clear_api_queue_unsigned(_a: u32, ) -> Weight {
		(6_141_000 as Weight)			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}}
