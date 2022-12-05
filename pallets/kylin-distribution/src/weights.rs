use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn create_distribution() -> Weight;
	fn add_recipient(x: u32) -> Weight;
	fn remove_recipient() -> Weight;
	fn enable_distribution() -> Weight;
	fn disable_distribution() -> Weight;
	fn claim(x: u32) -> Weight;
}

// SBP-M1 review: missing benchmarks for generated/hardcoded values

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn create_distribution() -> Weight {
		Weight::from_ref_time(66_168_000)
		.saturating_add(T::DbWeight::get().reads(3 as u64))
		.saturating_add(T::DbWeight::get().writes(3 as u64))
	}

	fn add_recipient(_x: u32) -> Weight {
		Weight::from_ref_time(66_168_000)
		.saturating_add(T::DbWeight::get().reads(3 as u64))
		.saturating_add(T::DbWeight::get().writes(3 as u64))
	}

	fn remove_recipient() -> Weight {
		Weight::from_ref_time(66_168_000)
		.saturating_add(T::DbWeight::get().reads(3 as u64))
		.saturating_add(T::DbWeight::get().writes(3 as u64))
	}

	fn enable_distribution() -> Weight {
		Weight::from_ref_time(66_168_000)
		.saturating_add(T::DbWeight::get().reads(3 as u64))
		.saturating_add(T::DbWeight::get().writes(3 as u64))
	}

	fn disable_distribution() -> Weight {
		Weight::from_ref_time(66_168_000)
		.saturating_add(T::DbWeight::get().reads(3 as u64))
		.saturating_add(T::DbWeight::get().writes(3 as u64))
	}

	fn claim(_x: u32) -> Weight {
		Weight::from_ref_time(66_168_000)
		.saturating_add(T::DbWeight::get().reads(3 as u64))
		.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
}

