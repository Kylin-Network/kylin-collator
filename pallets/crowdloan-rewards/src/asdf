#![cfg(feature = "runtime-benchmarks")]

use crate::{BalanceOf, Call, Pallet, WRAPPED_BYTES_PREFIX, WRAPPED_BYTES_POSTFIX};
use cumulus_pallet_parachain_system::Pallet as RelayPallet;
use cumulus_primitives_core::{
	relay_chain::{v1::HeadData, BlockNumber as RelayChainBlockNumber},
	PersistedValidationData,
};
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use ed25519_dalek::Signer;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
	dispatch::UnfilteredDispatchable,
	inherent::{InherentData, ProvideInherent},
	traits::{Currency, Get, OnFinalize, OnInitialize},
};
use frame_system::RawOrigin;
use parity_scale_codec::Encode;
use sp_core::{
	crypto::{AccountId32, UncheckedFrom},
	ed25519,
};
use sp_runtime::{traits::One, MultiSignature};
use sp_std::vec;
use sp_std::vec::Vec;
use sp_trie::StorageProof;
// This is a fake proof that emulates a storage proof inserted as the validation data
// We avoid using the sproof builder here because it generates an issue when compiling without std
// Fake storage proof
const MOCK_PROOF: [u8; 71] = [
	127, 1, 6, 222, 61, 138, 84, 210, 126, 68, 169, 213, 206, 24, 150, 24, 242, 45, 180, 180, 157,
	149, 50, 13, 144, 33, 153, 76, 133, 15, 37, 184, 227, 133, 144, 0, 0, 32, 0, 0, 0, 16, 0, 8, 0,
	0, 0, 0, 4, 0, 0, 0, 1, 0, 0, 5, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 6, 0, 0, 0,
];

// fake storage root. This is valid with the previous proof
const MOCK_PROOF_HASH: [u8; 32] = [
	216, 6, 227, 175, 180, 211, 98, 117, 202, 245, 206, 51, 21, 143, 100, 232, 96, 217, 14, 71,
	243, 146, 7, 202, 245, 129, 165, 70, 72, 184, 130, 141,
];

// These benchmarks only work with a Runtime that uses cumulus's RelayChainBlockNumberProvider.
// This will improve once https://github.com/PureStake/crowdloan-rewards/pull/44 lands
pub trait Config: crate::Config + cumulus_pallet_parachain_system::Config {}
impl<T: crate::Config + cumulus_pallet_parachain_system::Config> Config for T {}

/// Default balance amount is minimum contribution
fn default_balance<T: Config>() -> BalanceOf<T> {
	T::MinimumReward::get()
}

/// Create a funded user.
fn fund_specific_account<T: Config>(pallet_account: T::AccountId, extra: BalanceOf<T>) {
	let default_balance = default_balance::<T>();
	let total = default_balance + extra;
	T::RewardCurrency::make_free_balance_be(&pallet_account, total);
	T::RewardCurrency::issue(total);
}

/// Create a funded user.
fn create_funded_user<T: Config>(
	string: &'static str,
	n: u32,
	extra: BalanceOf<T>,
) -> T::AccountId {
	const SEED: u32 = 0;
	let user = account(string, n, SEED);
	let default_balance = default_balance::<T>();
	let total = default_balance + extra;
	T::RewardCurrency::make_free_balance_be(&user, total);
	T::RewardCurrency::issue(total);
	user
}

fn create_inherent_data<T: Config>(block_number: u32) -> InherentData {
	//let (relay_parent_storage_root, relay_chain_state) = create_fake_valid_proof();
	let vfp = PersistedValidationData {
		relay_parent_number: block_number as RelayChainBlockNumber,
		relay_parent_storage_root: MOCK_PROOF_HASH.into(),
		max_pov_size: 1000u32,
		parent_head: HeadData(vec![1, 1, 1]),
	};
	let inherent_data = {
		let mut inherent_data = InherentData::default();
		let system_inherent_data = ParachainInherentData {
			validation_data: vfp.clone(),
			relay_chain_state: StorageProof::new(vec![MOCK_PROOF.to_vec()]),
			downward_messages: Default::default(),
			horizontal_messages: Default::default(),
		};
		inherent_data
			.put_data(
				cumulus_primitives_parachain_inherent::INHERENT_IDENTIFIER,
				&system_inherent_data,
			)
			.expect("failed to put VFP inherent");
		inherent_data
	};
	inherent_data
}

/// Create contributors.
fn create_contributors<T: Config>(
	total_number: u32,
	seed_offset: u32,
) -> Vec<(T::RelayChainAccountId, Option<T::AccountId>, BalanceOf<T>)> {
	let mut contribution_vec = Vec::new();
	for i in 0..total_number {
		let seed = SEED - seed_offset - i;
		let mut account: [u8; 32] = [0u8; 32];
		let seed_as_slice = seed.to_be_bytes();
		for j in 0..seed_as_slice.len() {
			account[j] = seed_as_slice[j]
		}
		let relay_chain_account: AccountId32 = account.into();
		let user = create_funded_user::<T>("user", seed, 0u32.into());
		let contribution: BalanceOf<T> = 100u32.into();
		contribution_vec.push((relay_chain_account.into(), Some(user.clone()), contribution));
	}
	contribution_vec
}

/// Insert contributors.
fn insert_contributors<T: Config>(
	contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, BalanceOf<T>)>,
) -> Result<(), &'static str> {
	let mut sub_vec = Vec::new();
	let batch = max_batch_contributors::<T>();
	// Due to the MaxInitContributors associated type, we need ton insert them in batches
	// When we reach the batch size, we insert them
	for i in 0..contributors.len() {
		sub_vec.push(contributors[i].clone());
		// If we reached the batch size, we should insert them
		if i as u32 % batch == batch - 1 || i == contributors.len() - 1 {
			Pallet::<T>::initialize_reward_vec(RawOrigin::Root.into(), sub_vec.clone())?;
			sub_vec.clear()
		}
	}
	Ok(())
}

/// Create a Contributor.
fn close_initialization<T: Config>(
	end_vesting_block: T::VestingBlockNumber,
) -> Result<(), &'static str> {
	Pallet::<T>::complete_initialization(RawOrigin::Root.into(), end_vesting_block)?;
	Ok(())
}

fn create_sig<T: Config>(seed: u32, payload: Vec<u8>) -> (AccountId32, MultiSignature) {
	// Crate seed
	let mut seed_32: [u8; 32] = [0u8; 32];
	let seed_as_slice = seed.to_be_bytes();
	for j in 0..seed_as_slice.len() {
		seed_32[j] = seed_as_slice[j]
	}

	let secret = ed25519_dalek::SecretKey::from_bytes(&seed_32).unwrap();
	let public = ed25519_dalek::PublicKey::from(&secret);
	let pair = ed25519_dalek::Keypair { secret, public };
	let sig = pair.sign(&payload).to_bytes();
	let signature: MultiSignature = ed25519::Signature::from_raw(sig).into();

	let ed_public: ed25519::Public = ed25519::Public::unchecked_from(public.to_bytes());
	let account: AccountId32 = ed_public.into();
	(account, signature.into())
}

fn max_batch_contributors<T: Config>() -> u32 {
	T::MaxInitContributors::get()
}

// This is our current number of contributors
const MAX_ALREADY_USERS: u32 = 5799;
const SEED: u32 = 999999999;

benchmarks! {
	initialize_reward_vec {
		let x in 1..max_batch_contributors::<T>();
		let y = MAX_ALREADY_USERS;

		let total_pot = 100u32*(x+y);
		// We probably need to assume we have N contributors already in
		// Fund pallet account
		fund_specific_account::<T>(Pallet::<T>::account_id(), total_pot.into());

		// Create y contributors
		let contributors = create_contributors::<T>(y, 0);

		// Insert them
		insert_contributors::<T>(contributors)?;

		// This X new contributors are the ones we will count
		let new_contributors = create_contributors::<T>(x, y);

		let verifier = create_funded_user::<T>("user", SEED, 0u32.into());

	}:  _(RawOrigin::Root, new_contributors)
	verify {
		assert!(Pallet::<T>::accounts_payable(&verifier).is_some());
	}

	complete_initialization {
		// Fund pallet account
		let total_pot = 100u32;
		fund_specific_account::<T>(Pallet::<T>::account_id(), total_pot.into());
		// 1 contributor is enough
		let contributors = create_contributors::<T>(1, 0);

		// Insert them
		insert_contributors::<T>(contributors)?;

		// We need to create the first block inherent, to initialize the initRelayBlock
		let first_block_inherent = create_inherent_data::<T>(1u32);
		RelayPallet::<T>::on_initialize(T::BlockNumber::one());
		RelayPallet::<T>::create_inherent(&first_block_inherent)
			.expect("got an inherent")
			.dispatch_bypass_filter(RawOrigin::None.into())
			.expect("dispatch succeeded");
		RelayPallet::<T>::on_finalize(T::BlockNumber::one());
		Pallet::<T>::on_finalize(T::BlockNumber::one());

	}:  _(RawOrigin::Root, 10u32.into())
	verify {
	  assert!(Pallet::<T>::initialized());
	}

	claim {
		// Fund pallet account
		let total_pot = 100u32;
		fund_specific_account::<T>(Pallet::<T>::account_id(), total_pot.into());

		// The user that will make the call
		let caller: T::AccountId = create_funded_user::<T>("user", SEED, 100u32.into());

		// We verified there is no dependency of the number of contributors already inserted in claim
		// Create 1 contributor
		let contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, BalanceOf<T>)> =
			vec![(AccountId32::from([1u8;32]).into(), Some(caller.clone()), total_pot.into())];

		// Insert them
		insert_contributors::<T>(contributors)?;

		// Close initialization
		close_initialization::<T>(10u32.into())?;

		// First inherent
		let first_block_inherent = create_inherent_data::<T>(1u32);
		RelayPallet::<T>::on_initialize(T::BlockNumber::one());
		RelayPallet::<T>::create_inherent(&first_block_inherent)
			.expect("got an inherent")
			.dispatch_bypass_filter(RawOrigin::None.into())
			.expect("dispatch succeeded");
		RelayPallet::<T>::on_finalize(T::BlockNumber::one());
		Pallet::<T>::on_finalize(T::BlockNumber::one());

		// Create 4th relay block, by now the user should have vested some amount
		RelayPallet::<T>::on_initialize(4u32.into());

		let last_block_inherent = create_inherent_data::<T>(4u32);
		RelayPallet::<T>::create_inherent(&last_block_inherent)
			.expect("got an inherent")
			.dispatch_bypass_filter(RawOrigin::None.into())
			.expect("dispatch succeeded");

		RelayPallet::<T>::on_finalize(4u32.into());

	}:  _(RawOrigin::Signed(caller.clone()))
	verify {
	  assert_eq!(Pallet::<T>::accounts_payable(&caller).unwrap().total_reward, (100u32.into()));
	}

	update_reward_address {
		// Fund pallet account
		let total_pot = 100u32;
		fund_specific_account::<T>(Pallet::<T>::account_id(), total_pot.into());

		// The user that will make the call
		let caller: T::AccountId = create_funded_user::<T>("user", SEED, 100u32.into());

		let relay_account: T::RelayChainAccountId = AccountId32::from([1u8;32]).into();
		// We verified there is no dependency of the number of contributors already inserted in update_reward_address
		// Create 1 contributor
		let contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, BalanceOf<T>)> =
			vec![(relay_account.clone(), Some(caller.clone()), total_pot.into())];

		// Insert them
		insert_contributors::<T>(contributors)?;

		// Close initialization
		close_initialization::<T>(10u32.into())?;

		// First inherent
		let first_block_inherent = create_inherent_data::<T>(1u32);
		RelayPallet::<T>::on_initialize(T::BlockNumber::one());
		RelayPallet::<T>::create_inherent(&first_block_inherent)
			.expect("got an inherent")
			.dispatch_bypass_filter(RawOrigin::None.into())
			.expect("dispatch succeeded");
		RelayPallet::<T>::on_finalize(T::BlockNumber::one());
		Pallet::<T>::on_finalize(T::BlockNumber::one());


		// Let's advance the relay so that the vested  amount get transferred

		RelayPallet::<T>::on_initialize(4u32.into());
		let last_block_inherent = create_inherent_data::<T>(4u32);
		RelayPallet::<T>::create_inherent(&last_block_inherent)
			.expect("got an inherent")
			.dispatch_bypass_filter(RawOrigin::None.into())
			.expect("dispatch succeeded");

		RelayPallet::<T>::on_finalize(4u32.into());

		// The new user
		let new_user = create_funded_user::<T>("user", SEED+1, 0u32.into());

	}:  _(RawOrigin::Signed(caller.clone()), new_user.clone())
	verify {
		assert_eq!(Pallet::<T>::accounts_payable(&new_user).unwrap().total_reward, (100u32.into()));
		assert!(Pallet::<T>::claimed_relay_chain_ids(&relay_account).is_some());
	}

	associate_native_identity {
		// Fund pallet account
		let total_pot = 100u32;
		fund_specific_account::<T>(Pallet::<T>::account_id(), total_pot.into());

		// The caller that will associate the account
		let caller: T::AccountId = create_funded_user::<T>("user", SEED, 100u32.into());

		// Create a fake sig for such an account
		let (relay_account, signature) = create_sig::<T>(SEED, caller.clone().encode());

		// We verified there is no dependency of the number of contributors already inserted in associate_native_identity
		// Create 1 contributor
		let contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, BalanceOf<T>)> =
		vec![(relay_account.clone().into(), None, total_pot.into())];

		// Insert them
		insert_contributors::<T>(contributors)?;

		// Clonse initialization
		close_initialization::<T>(10u32.into())?;

		// First inherent
		let first_block_inherent = create_inherent_data::<T>(1u32);
		RelayPallet::<T>::on_initialize(T::BlockNumber::one());
		RelayPallet::<T>::create_inherent(&first_block_inherent)
			.expect("got an inherent")
			.dispatch_bypass_filter(RawOrigin::None.into())
			.expect("dispatch succeeded");
		RelayPallet::<T>::on_finalize(T::BlockNumber::one());
		Pallet::<T>::on_finalize(T::BlockNumber::one());

	}:  _(RawOrigin::Signed(caller.clone()), caller.clone(), relay_account.into(), signature)
	verify {
		assert_eq!(Pallet::<T>::accounts_payable(&caller).unwrap().total_reward, (100u32.into()));
	}

	change_association_with_relay_keys {

		// The weight will depend on the number of proofs provided
		// We need to parameterize this value
		// We leave this as the max batch length
		let x in 1..max_batch_contributors::<T>();

		// Fund pallet account
		let total_pot = 100u32*x;
		fund_specific_account::<T>(Pallet::<T>::account_id(), total_pot.into());

		// The first reward account that will associate the account
		let first_reward_account: T::AccountId = create_funded_user::<T>("user", SEED, 100u32.into());

		// The account to which we will update our reward account
		let second_reward_account: T::AccountId = create_funded_user::<T>("user", SEED-1, 100u32.into());

		let mut proofs: Vec<(T::RelayChainAccountId, MultiSignature)> = Vec::new();

		// Construct payload
		let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
		payload.append(&mut second_reward_account.clone().encode());
		payload.append(&mut first_reward_account.clone().encode());
		payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());

		// Create N sigs for N accounts
		for i in 0..x {
			let (relay_account, signature) = create_sig::<T>(SEED-i, payload.clone());
			proofs.push((relay_account.into(), signature));
		}

		// Create x contributors
		// All of them map to the same account
		let mut contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, BalanceOf<T>)> = Vec::new();
		for (relay_account, _) in proofs.clone() {
			contributors.push((relay_account, Some(first_reward_account.clone()), 100u32.into()));
		}

		// Insert them
		insert_contributors::<T>(contributors.clone())?;

		// Clonse initialization
		close_initialization::<T>(10u32.into())?;

		// First inherent
		let first_block_inherent = create_inherent_data::<T>(1u32);
		RelayPallet::<T>::on_initialize(T::BlockNumber::one());
		RelayPallet::<T>::create_inherent(&first_block_inherent)
			.expect("got an inherent")
			.dispatch_bypass_filter(RawOrigin::None.into())
			.expect("dispatch succeeded");
		RelayPallet::<T>::on_finalize(T::BlockNumber::one());
		Pallet::<T>::on_finalize(T::BlockNumber::one());

	}:  _(RawOrigin::Signed(first_reward_account.clone()), second_reward_account.clone(), first_reward_account.clone(), proofs)
	verify {
		assert!(Pallet::<T>::accounts_payable(&second_reward_account).is_some());
		assert_eq!(Pallet::<T>::accounts_payable(&second_reward_account).unwrap().total_reward, (100u32*x).into());
		assert!(Pallet::<T>::accounts_payable(&first_reward_account).is_none());

	}

}
#[cfg(test)]
mod tests {
	use crate::mock::Test;
	use sp_io::TestExternalities;

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		TestExternalities::new(t)
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Test
);
