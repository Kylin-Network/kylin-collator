
#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for module_evm_accounts.
pub trait WeightInfo {
	fn claim_account() -> Weight;
	fn claim_default_account() -> Weight;
}

/// Weights for module_evm_accounts using the Kylin node and recommended hardware.
pub struct KylinWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for KylinWeight<T> {
	fn claim_account() -> Weight {
		(340_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn claim_default_account() -> Weight {
		(19_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn claim_account() -> Weight {
		(340_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn claim_default_account() -> Weight {
		(19_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
}
