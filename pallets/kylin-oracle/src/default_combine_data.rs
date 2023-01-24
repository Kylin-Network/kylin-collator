use crate::{Config, TimestampedValueT, OracleKeyOf};
use frame_support::traits::{Get, UnixTime};
use orml_traits::CombineData;
use sp_std::{marker, prelude::*};
use hex::ToHex;

/// Sort by value and returns median timestamped value.
/// Returns prev_value if not enough valid values.
pub struct DefaultCombineData<T, MinimumCount, ExpiresIn>(marker::PhantomData<(T, MinimumCount, ExpiresIn)>);

impl<T,MinimumCount, ExpiresIn> CombineData<OracleKeyOf<T>, TimestampedValueT>
	for DefaultCombineData<T, MinimumCount, ExpiresIn>
where
	T: Config,
	T::AccountId: AsRef<[u8]> + ToHex,
	MinimumCount: Get<u32>,
	ExpiresIn: Get<u128>,
{
	fn combine_data(
		_key: &OracleKeyOf<T>,
		mut values: Vec<TimestampedValueT>,
		prev_value: Option<TimestampedValueT>,
	) -> Option<TimestampedValueT> {
		let expires_in = ExpiresIn::get();
		let now = T::UnixTime::now().as_millis();

		values.retain(|x| x.timestamp + expires_in > now);

		let count = values.len() as u32;
		let minimum_count = MinimumCount::get();
		if count < minimum_count || count == 0 {
			return prev_value;
		}

		let mid_index = count / 2;
		// Won't panic as `values` ensured not empty.
		let (_, value, _) = values.select_nth_unstable_by(mid_index as usize, |a, b| a.value.cmp(&b.value));
		Some(value.clone())
	}
}
