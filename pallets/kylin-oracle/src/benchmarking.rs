#![cfg(feature = "runtime-benchmarks")]
use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};

benchmarks! {
    where_clause { where
		T::AccountId: AsRef<[u8]>,
	}

    sudo_remove_feed_account {
        let a in 1 .. 100;
        let caller: T::AccountId = whitelisted_caller();
		let feed_name = str::from_utf8(b"price_feeding").unwrap().as_bytes().to_vec();
        let hash: Vec<u8> = Vec::new();
        <FeedAccountLookup<T>>::insert(feed_name.clone(), (caller.clone(), hash));
    }: _(RawOrigin::Root, feed_name.clone())
    verify {
        assert_last_event::<T>(Event::RemovedFeedAccount(feed_name).into());
    }

    submit_price_feed { 
        let a in 1 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let api_url = str::from_utf8(b"https://api.kylin-node.co.uk/prices?currency_pairs=").unwrap();
        let requested_currencies = str::from_utf8(b"btc_usd,eth_gdp").unwrap();
        let url = api_url.clone().to_owned() + requested_currencies;
    }: _(RawOrigin::Signed(caller.clone()), None, requested_currencies.as_bytes().to_vec())
    verify {
        let block_number = frame_system::Pallet::<T>::block_number();
        let feed_name = str::from_utf8(b"price_feeding").unwrap().as_bytes().to_vec();
        assert_last_event::<T>(Event::SubmitNewData(
            None,
            feed_name,
            Some(url.as_bytes().to_vec()),
            Some(caller),
            block_number).into()
        );
    }

    query_data {
        let a in 1 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let _ = T::Currency::make_free_balance_be(&caller, T::Currency::minimum_balance());
        let feed_name = str::from_utf8(b"custom_benchmark_feed").unwrap().as_bytes().to_vec();
        let data = b"{'benchmark':'this is data'}".to_vec();
        // Submit data signed
        let key: u64 = a.into();
        let block_number = <system::Pallet<T>>::block_number();
        let data = str::from_utf8(b"{'benchmark':'this is data'}").unwrap().as_bytes().to_vec();
        <Pallet<T>>::submit_data_signed(RawOrigin::Signed(caller.clone()).into(), block_number, key, data.clone()).unwrap();
        // Clear_processed_requests_unsigned 
        <Pallet<T>>::clear_processed_requests_unsigned(RawOrigin::None.into(), block_number, [key].to_vec()).unwrap();
        // Clear api queue
        <Pallet<T>>::clear_api_queue_unsigned(RawOrigin::None.into(), block_number, [key].to_vec()).unwrap();
        // Add fresh data
        <Pallet<T>>::submit_data_signed(RawOrigin::Signed(caller.clone()).into(), block_number, key, data).unwrap();
        // Insert new key to feed account lookup
        let hash: Vec<u8> = str::from_utf8(b"5F28xL42VWThNonDft4TAQ6rw6a82E2jMsQXS5uMyKiA4ccv").unwrap().as_bytes().to_vec();
        <FeedAccountLookup<T>>::insert(feed_name.clone(), (caller.clone(), hash));
        // Get new feed name
        // let new_feed_name = str::from_utf8(b"custom_").unwrap().to_owned() + str::from_utf8(&feed_name).unwrap();
    }: _(RawOrigin::Signed(caller), None, feed_name)

    write_data_onchain {
        let a in 1 .. 100;
        let caller: T::AccountId = whitelisted_caller();
		let feed_name = str::from_utf8(b"feed_name").unwrap().as_bytes().to_vec();
        let data = b"{'benchmark':'this is data'}".to_vec();
    }: _(RawOrigin::Signed(caller.clone()), feed_name.clone(), data)
    verify {
        let block_number = frame_system::Pallet::<T>::block_number();
        let new_feed_name = str::from_utf8(b"custom_").unwrap().to_owned() + str::from_utf8(&feed_name).unwrap();
        assert_last_event::<T>(Event::SubmitNewData(
            None,
            new_feed_name.as_bytes().to_vec(),
            None,
            Some(caller.clone()),
            block_number).into(),
        );
    }

    submit_data_signed {
        let a in 1 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let block_number = <system::Pallet<T>>::block_number();
        let key: u64 = a.into();
        let data = str::from_utf8(b"{'benchmark':'this is data'}").unwrap().as_bytes().to_vec();
    }: _(RawOrigin::Signed(caller), block_number, key, data.clone())
    verify {
        let saved_request = Pallet::<T>::saved_data_requests(key.clone());
        assert_eq!(saved_request.payload, data)
    }

    submit_data_unsigned {
        let a in 1 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let block_number = <system::Pallet<T>>::block_number();
        let key: u64 = a.into();
        let data = str::from_utf8(b"{'benchmark':'this is data'}").unwrap().as_bytes().to_vec();
    }: _(RawOrigin::None, block_number, key, data.clone())
    verify {
        let saved_request = Pallet::<T>::saved_data_requests(key.clone());
        assert_eq!(saved_request.payload, data)
    }

    submit_data_via_api {
        let a in 1 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let url = str::from_utf8(b"https://min-api.cryptocompare.com/data/price?fsym=btc&tsyms=usd").unwrap().as_bytes().to_vec();
		let feed_name = str::from_utf8(b"feed_name").unwrap().as_bytes().to_vec();
    }: _(RawOrigin::Signed(caller.clone()), None, url.clone(), feed_name.clone())
    verify {
        let block_number = frame_system::Pallet::<T>::block_number();
        let new_feed_name = str::from_utf8(b"custom_").unwrap().to_owned() + str::from_utf8(&feed_name).unwrap();
        assert_last_event::<T>(Event::SubmitNewData(
            None,
            new_feed_name.as_bytes().to_vec(),
            Some(url),
            Some(caller),
            block_number).into()
        );
    }

    // xcm_submit_data_via_api {
    //     let a in 1 .. 100;
    //     let caller: T::AccountId = whitelisted_caller();
    //     let url = str::from_utf8(b"https://min-api.cryptocompare.com/data/price?fsym=btc&tsyms=usd").unwrap().as_bytes().to_vec();
	// 	let feed_name = str::from_utf8(b"feed_name").unwrap().as_bytes().to_vec();
    // }: _(RawOrigin::Root, url, feed_name)

    // receive_response_from_parachain {
    //     let a in 1 .. 100;
    //     let caller: T::AccountId = whitelisted_caller();
	// 	let feed_name = str::from_utf8(b"feed_name").unwrap().as_bytes().to_vec();
    //     let response = b"{'test':'test'}".to_vec();
    // }: _(RawOrigin::Root, feed_name, response)

    clear_api_queue_unsigned {
        let a in 1 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let block_number = <system::Pallet<T>>::block_number();

        let key: u64 = a.into();
        let saved_request = DataRequest {
            para_id: None,
            account_id: Some(caller),
            feed_name: str::from_utf8(b"price_feeding").unwrap().as_bytes().to_vec(),
            requested_block_number: frame_system::Pallet::<T>::block_number(),
            processed_block_number: None,
            requested_timestamp: T::UnixTime::now().as_millis(),
            processed_timestamp: None,
            payload: str::from_utf8(b"{'benchmark':'this is data'}").unwrap().as_bytes().to_vec(),
            is_query: false,
            url: None,
        };
        <ApiQueue<T>>::insert(key.clone(), saved_request.clone());
        let processed_requests = [key.clone()].to_vec();

    }: _(RawOrigin::None, block_number, processed_requests)
    verify {
        let in_queue = ApiQueue::<T>::contains_key(key);
        assert_eq!(in_queue, false);
    }
}

impl_benchmark_test_suite!(
	Pallet,
	crate::mock::TestExternalitiesBuilder::default().build(|| {}),
	crate::mock::MockRuntime,
);


fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) where <T as frame_system::Config>::AccountId: AsRef<[u8]> {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}
