#![cfg(feature = "runtime-benchmarks")]
use super::*;
use crate::*;
use pallet::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

impl<T: Config> Pallet<T>
where
    T::AccountId: AsRef<[u8]> + ToHex,
    T: pallet_balances::Config,
{}

benchmarks! {
    write_data_onchain {
        let a in 1 .. 100;
        let feed_name = b"feed_name".to_vec();
        let data = b"{'test':'test'}".to_vec();
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::signed(caller), feed_name, data)
    verify {
        assert_eq!(1,1);
        // assert_eq!(Pallet::<T>::get(), Some(a.into));
    }
}

impl_benchmark_test_suite!(
    Pallet,
    crate::mock::TestExternalitiesBuilder::default().build(|| {}),
    crate::mock::MockRuntime,
);