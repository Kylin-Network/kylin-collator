#![cfg(feature = "runtime-benchmarks")]
use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

// fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
// 	let events = frame_system::Pallet::<T>::events();
// 	let system_event: <T as frame_system::Config>::Event = generic_event.into();
// 	// compare to the last event record
// 	let EventRecord { event, .. } = &events[events.len() - 1];
// 	assert_eq!(event, &system_event);
// }

benchmarks! {
    where_clause { where
		T::AccountId: AsRef<[u8]>,
	}

    write_data_onchain {
        let a in 1 .. 100;
        let feed_name = a.to_string();
        let feed_name_vec = feed_name.as_bytes().to_vec();
        let data = b"{'test':'test'}".to_vec();
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), feed_name_vec, data.clone())
    verify {
        // assert_last_event::<T>(Event::SubmitNewData(None, feed_name_vec, None, caller, frame_system::Pallet::<T>::block_number()).into());
        let key = DataId::<T>::get();
        let data_request = Pallet::<T>::data_requests(key);
        assert_eq!(data_request.payload, data);
    }
}

impl_benchmark_test_suite!(
    Pallet,
    crate::mock::TestExternalitiesBuilder::default().build(|| {}),
    crate::mock::MockRuntime,
);