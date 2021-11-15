//! Benchmarking setup for pallet-kylin-oracle
#[cfg(feature = "runtime-benchmarks")]
use super::*;
use crate as kylin_oracle;
use crate::*;
use hex::ToHex;
#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use frame_system::EventRecord;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	

/*fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}*/

benchmarks! {
	where_clause { where 
		T::AccountId: AsRef<[u8]>,	
	}
	submit_data_unsigned {
		let s in 0 .. 100;
		
        let caller: T::AccountId = whitelisted_caller();		
		let benchmarking_test_data = str::from_utf8(b"Testing Benchmark").unwrap().as_bytes().to_vec();
		let block_number = <system::Pallet<T>>::block_number();
		
    }: _(RawOrigin::None, block_number, 18446744073709551615, benchmarking_test_data)      
	verify {
		//assert_eq!(Something::<T>::get(), Some(s));
		//assert_last_event(Event::ResponseSent);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::{ExtBuilder, Test};
	use frame_Support::assert_ok;

	#[test]
	fn submit_data_signed (){
		let mut t = sp_io::TestExternalities::default();
		t.execute_with(||{
			assert_ok!(test_benchmark_submit_data_signed::<Test>())
		})
	}

}

impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test,);
