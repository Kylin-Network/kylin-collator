// Copyright 2019-2020 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! # Crowdloan Rewards Pallet
//!
//! This pallet issues rewards to citizens who participated in a crowdloan on the backing relay
//! chain (eg Kusama) in order to help this parrachain acquire a parachain slot.
//!
//! ## Monetary Policy
//!
//! This is simple and mock for now. We can do whatever we want.
//! This pallet stores a constant  "reward ratio" which is the number of reward tokens to pay per
//! contributed token. In simple cases this can be 1, but needs to be customizeable to allow for
//! vastly differing absolute token supplies between relay and para.
//! Vesting is also linear. No tokens are vested at genesis and they unlock linearly until a
//! predecided block number. Vesting computations happen on demand when payouts are requested. So
//! no block weight is ever wasted on this, and there is no "base-line" cost of updating vestings.
//! Like I said, we can anything we want there. Even a non-linear reward curve to disincentivize
//! whales.
//!
//! ## Payout Mechanism
//!
//! The current payout mechanism requires contributors to claim their payouts. Because they are
//! paying the transaction fees for this themselves, they can do it as often as every block, or
//! wait and claim the entire thing once it is fully vested. We could consider auto payouts if we
//! want.
//!
//! ## Sourcing Contribution Information
//!
//! The pallet can learn about the crowdloan contributions in several ways.
//!
//! * **Through the initialize_reward_vec extrinsic*
//!
//! The simplest way is to call the initialize_reward_vec through a democracy proposal/sudo call.
//! This makes sense in a scenario where the crowdloan took place entirely offchain.
//! This extrinsic initializes the associated and unassociated stoerage with the provided data
//!
//! * **ReadingRelayState**
//!
//! The most elegant, but most complex solution would be for the para to read the contributions
//! directly from the relay state. Blocked by https://github.com/paritytech/cumulus/issues/320 so
//! I won't pursue it further right now. I can't decide whether that would really add security /
//! trustlessness, or is just a sexy blockchain thing to do. Contributors can always audit the
//! democracy proposal and make sure their contribution is in it, so in that sense reading relay state
//! isn't necessary. But if a single contribution is left out, the rest of the contributors might
//! not care enough to delay network launch. The little guy might get censored.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;
pub mod weights;


use crate::weights::WeightInfo;
use frame_support::traits::WithdrawReasons;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement::AllowDeath},
	PalletId,
};
use frame_system::pallet_prelude::*;
use sp_core::crypto::AccountId32;
use sp_runtime::traits::{
	AccountIdConversion, AtLeast32BitUnsigned, BlockNumberProvider, Saturating,
};
use sp_runtime::{MultiSignature, Perbill};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::vec;
use sp_std::vec::Vec;
use scale_info::TypeInfo;
use sp_std::fmt::Debug;
#[frame_support::pallet]
pub mod pallet {
    use super::*;



	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	pub const PALLET_ID: PalletId = PalletId(*b"Crwdloan");

	// The wrapper around which the reward changing message needs to be wrapped
	pub const WRAPPED_BYTES_PREFIX: &[u8] = b"<Bytes>";
	pub const WRAPPED_BYTES_POSTFIX: &[u8] = b"</Bytes>";

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Checker for the reward vec, is it initalized already?
		type Initialized: Get<bool>;
		/// Percentage to be payed at initialization
		#[pallet::constant]
		type InitializationPayment: Get<Perbill>;
		// Max number of contributors that can be inserted at once in initialize_reward_vec
		#[pallet::constant]
		type MaxInitContributors: Get<u32>;
		/// The minimum contribution to which rewards will be paid.
		type MinimumReward: Get<BalanceOf<Self>>;
		/// A fraction representing the percentage of proofs
		/// that need to be presented to change a reward address through the relay keys
		#[pallet::constant]
		type RewardAddressRelayVoteThreshold: Get<Perbill>;
		/// The currency in which the rewards will be paid (probably the parachain native currency)
		type RewardCurrency: Currency<Self::AccountId>;
		/// The AccountId type contributors used on the relay chain.
		type RelayChainAccountId: Parameter
			//TODO these AccountId32 bounds feel a little extraneous. I wonder if we can remove them.
			+ Into<AccountId32>
			+ From<AccountId32>
			+ TypeInfo
			+ Ord;

		// The origin that is allowed to change the reward address with relay signatures
		type RewardAddressChangeOrigin: EnsureOrigin<Self::Origin>;

		/// The type that will be used to track vesting progress
		type VestingBlockNumber: AtLeast32BitUnsigned + Parameter + Default + Into<BalanceOf<Self>>;

		/// The notion of time that will be used for vesting. Probably
		/// either the relay chain or sovereign chain block number.
		type VestingBlockProvider: BlockNumberProvider<BlockNumber = Self::VestingBlockNumber>;

		type WeightInfo: WeightInfo;
	}



	// This hook is in charge of initializing the vesting height at the first block of the parachain
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: <T as frame_system::Config>::BlockNumber) {
			// In the first block of the parachain we need to introduce the vesting block related info
			if n == 1u32.into() {
				<InitVestingBlock<T>>::put(T::VestingBlockProvider::current_block_number());
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Associate a native rewards_destination identity with a crowdloan contribution.
		///
		/// The caller needs to provide the unassociated relay account and a proof to succeed
		/// with the association
		/// The proof is nothing but a signature over the reward_address using the relay keys
		#[pallet::weight(T::WeightInfo::associate_native_identity())]
		pub fn associate_native_identity(
			origin: OriginFor<T>,
			reward_account: T::AccountId,
			relay_account: T::RelayChainAccountId,
			proof: MultiSignature,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			// Check the proof:
			// 1. Is signed by an actual unassociated contributor
			// 2. Signs a valid native identity
			// Check the proof. The Proof consists of a Signature of the rewarded account with the
			// claimer key

			// The less costly checks will go first

			// The relay account should be unassociated
			let mut reward_info = UnassociatedContributions::<T>::get(&relay_account)
				.ok_or(Error::<T>::NoAssociatedClaim)?;

			// We ensure the relay chain id wast not yet associated to avoid multi-claiming
			// We dont need this right now, as it will always be true if the above check is true
			ensure!(
				ClaimedRelayChainIds::<T>::get(&relay_account).is_none(),
				Error::<T>::AlreadyAssociated
			);

			// For now I prefer that we dont support providing an existing account here
			ensure!(
				AccountsPayable::<T>::get(&reward_account).is_none(),
				Error::<T>::AlreadyAssociated
			);

			let payload = reward_account.encode();

			// Check the signature
			Self::verify_signatures(
				vec![(relay_account.clone(), proof)],
				reward_info.clone(),
				payload,
			)?;

			// Make the first payment
			let first_payment = T::InitializationPayment::get() * reward_info.total_reward;

			T::RewardCurrency::transfer(
				&PALLET_ID.into_account(),
				&reward_account,
				first_payment,
				AllowDeath,
			)?;

			Self::deposit_event(Event::InitialPaymentMade(
				reward_account.clone(),
				first_payment,
			));

			reward_info.claimed_reward = first_payment;

			// Insert on payable
			AccountsPayable::<T>::insert(&reward_account, &reward_info);

			// Remove from unassociated
			<UnassociatedContributions<T>>::remove(&relay_account);

			// Insert in mapping
			ClaimedRelayChainIds::<T>::insert(&relay_account, ());

			// Emit Event
			Self::deposit_event(Event::NativeIdentityAssociated(
				relay_account,
				reward_account,
				reward_info.total_reward,
			));

			Ok(Default::default())
		}

		/// Change reward account by submitting proofs from relay accounts
		///
		/// The number of valid proofs needs to be bigger than 'RewardAddressRelayVoteThreshold'
		/// The account to be changed needs to be submitted as 'previous_account'

		/// Origin must be RewardAddressChangeOrigin
		#[pallet::weight(T::WeightInfo::change_association_with_relay_keys(proofs.len() as u32))]
		pub fn change_association_with_relay_keys(
			origin: OriginFor<T>,
			reward_account: T::AccountId,
			previous_account: T::AccountId,
			proofs: Vec<(T::RelayChainAccountId, MultiSignature)>,
		) -> DispatchResultWithPostInfo {
			// Check that the origin is the one able to change the reward addrss
			T::RewardAddressChangeOrigin::ensure_origin(origin)?;

			// For now I prefer that we dont support providing an existing account here
			ensure!(
				AccountsPayable::<T>::get(&reward_account).is_none(),
				Error::<T>::AlreadyAssociated
			);

			// To avoid replay attacks, we make sure the payload contains the previous address too
			// I am assuming no rational user will go back to a previously changed reward address
			// b"<Bytes>" + "new_account" + "previous_account" + b"</Bytes>"
			let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
			payload.append(&mut reward_account.encode());
			payload.append(&mut previous_account.encode());
			payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());

			// Get the reward info for the account to be changed
			let reward_info = AccountsPayable::<T>::get(&previous_account)
				.ok_or(Error::<T>::NoAssociatedClaim)?;

			Self::verify_signatures(proofs, reward_info.clone(), payload)?;

			// Remove fromon payable
			AccountsPayable::<T>::remove(&previous_account);

			// Insert on payable
			AccountsPayable::<T>::insert(&reward_account, &reward_info);

			// Emit Event
			Self::deposit_event(Event::RewardAddressUpdated(
				previous_account,
				reward_account,
			));

			Ok(Default::default())
		}

		/// Collect whatever portion of your reward are currently vested.
		#[pallet::weight(T::WeightInfo::claim())]
		pub fn claim(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let payee = ensure_signed(origin)?;
			let initialized = <Initialized<T>>::get();
			ensure!(initialized, Error::<T>::RewardVecNotFullyInitializedYet);
			// Calculate the veted amount on demand.
			let mut info =
				AccountsPayable::<T>::get(&payee).ok_or(Error::<T>::NoAssociatedClaim)?;
			ensure!(
				info.claimed_reward < info.total_reward,
				Error::<T>::RewardsAlreadyClaimed
			);

			// Get the current block used for vesting purposes
			let now = T::VestingBlockProvider::current_block_number();

			// Substract the first payment from the vested amount
			let first_paid = T::InitializationPayment::get() * info.total_reward;

			// To calculate how much could the user have claimed already
			let payable_period = now.saturating_sub(<InitVestingBlock<T>>::get());

			// How much should the contributor have already claimed by this block?
			// By multiplying first we allow the conversion to integer done with the biggest number
			let period = EndVestingBlock::<T>::get() - InitVestingBlock::<T>::get();
			let should_have_claimed = if period == 0u32.into() {
				// Pallet is configured with a zero vesting period.
				info.total_reward - first_paid
			} else {
				(info.total_reward - first_paid).saturating_mul(payable_period.into())
					/ period.into()
			};

			// If the period is bigger than whats missing to pay, then return whats missing to pay
			let payable_amount = if should_have_claimed >= (info.total_reward - first_paid) {
				info.total_reward.saturating_sub(info.claimed_reward)
			} else {
				should_have_claimed + first_paid - info.claimed_reward
			};

			info.claimed_reward = info.claimed_reward.saturating_add(payable_amount);
			AccountsPayable::<T>::insert(&payee, &info);

			// This pallet controls an amount of funds and transfers them to each of the contributors
			//TODO: contributors should have the balance locked for tranfers but not for democracy
			T::RewardCurrency::transfer(
				&PALLET_ID.into_account(),
				&payee,
				payable_amount,
				AllowDeath,
			)?;
			// Emit event
			Self::deposit_event(Event::RewardsPaid(payee, payable_amount));
			Ok(Default::default())
		}

		/// Update reward address, proving that the caller owns the current native key
		#[pallet::weight(T::WeightInfo::update_reward_address())]
		pub fn update_reward_address(
			origin: OriginFor<T>,
			new_reward_account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let signer = ensure_signed(origin)?;

			// Calculate the veted amount on demand.
			let info = AccountsPayable::<T>::get(&signer).ok_or(Error::<T>::NoAssociatedClaim)?;

			// For now I prefer that we dont support providing an existing account here
			ensure!(
				AccountsPayable::<T>::get(&new_reward_account).is_none(),
				Error::<T>::AlreadyAssociated
			);

			// Remove previous rewarded account
			AccountsPayable::<T>::remove(&signer);

			// Update new rewarded acount
			AccountsPayable::<T>::insert(&new_reward_account, &info);

			// Emit event
			Self::deposit_event(Event::RewardAddressUpdated(signer, new_reward_account));

			Ok(Default::default())
		}

		/// Initialize the reward distribution storage. It shortcuts whenever an error is found
		/// This does not enforce any checks other than making sure we dont go over funds
		/// complete_initialization should perform any additional
		#[pallet::weight(T::WeightInfo::initialize_reward_vec(rewards.len() as u32))]
		pub fn initialize_reward_vec(
			origin: OriginFor<T>,
			rewards: Vec<(T::RelayChainAccountId, Option<T::AccountId>, BalanceOf<T>)>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let initialized = <Initialized<T>>::get();
			ensure!(
				initialized == false,
				Error::<T>::RewardVecAlreadyInitialized
			);

			// Ensure we are below the max number of contributors
			ensure!(
				rewards.len() as u32 <= T::MaxInitContributors::get(),
				Error::<T>::TooManyContributors
			);

			// What is the amount initialized so far?
			let mut current_initialized_rewards = InitializedRewardAmount::<T>::get();

			// Total number of contributors
			let mut total_contributors = TotalContributors::<T>::get();

			let incoming_rewards: BalanceOf<T> = rewards
				.iter()
				.fold(0u32.into(), |acc: BalanceOf<T>, (_, _, reward)| {
					acc + *reward
				});


			// Ensure we dont go over funds
			ensure!(
				current_initialized_rewards + incoming_rewards <= Self::pot(),
				Error::<T>::BatchBeyondFundPot
			);


			for (relay_account, native_account, reward) in &rewards {
				if ClaimedRelayChainIds::<T>::get(&relay_account).is_some()
					|| UnassociatedContributions::<T>::get(&relay_account).is_some()
				{

					// Dont fail as this is supposed to be called with batch calls and we
					// dont want to stall the rest of the contributions
					Self::deposit_event(Event::InitializedAlreadyInitializedAccount(
						relay_account.clone(),
						native_account.clone(),
						*reward,
					));
					continue;
				} 
			
				if *reward < T::MinimumReward::get() {

					// Don't fail as this is supposed to be called with batch calls and we
					// dont want to stall the rest of the contributions
					Self::deposit_event(Event::InitializedAccountWithNotEnoughContribution(
						relay_account.clone(),
						native_account.clone(),
						*reward,
					));
					continue;
				} 

				// If we have a native_account, we make the payment
				let initial_payment = if let Some(native_account) = native_account {

					let first_payment = T::InitializationPayment::get() * (*reward);
					T::RewardCurrency::transfer(
						&PALLET_ID.into_account(),
						&native_account,
						first_payment,
						AllowDeath,
					)?;
					Self::deposit_event(Event::InitialPaymentMade(
						native_account.clone(),
						first_payment,
					));
					first_payment
				} else {
					0u32.into()
				};

				// Calculate the reward info to store after the initial payment has been made.
				let mut reward_info = RewardInfo {
					total_reward: *reward,
					claimed_reward: initial_payment,
					contributed_relay_addresses: vec![relay_account.clone()],
				};

				current_initialized_rewards += *reward - initial_payment;
				total_contributors += 1;

				if let Some(native_account) = native_account {
					if let Some(mut inserted_reward_info) =
						AccountsPayable::<T>::get(native_account)
					{

						inserted_reward_info
							.contributed_relay_addresses
							.append(&mut reward_info.contributed_relay_addresses);
						// the native account has already some rewards in, we add the new ones
						AccountsPayable::<T>::insert(
							native_account,
							RewardInfo {
								total_reward: inserted_reward_info.total_reward
									+ reward_info.total_reward,
								claimed_reward: inserted_reward_info.claimed_reward
									+ reward_info.claimed_reward,
								contributed_relay_addresses: inserted_reward_info
									.contributed_relay_addresses,
							},
						);
					} else {
						// First reward association
						AccountsPayable::<T>::insert(native_account, reward_info);
					}
					ClaimedRelayChainIds::<T>::insert(relay_account, ());
				} else {
					UnassociatedContributions::<T>::insert(relay_account, reward_info);
				}
			}


			InitializedRewardAmount::<T>::put(current_initialized_rewards);

			TotalContributors::<T>::put(total_contributors);

			Ok(Default::default())
		}


		/// This extrinsic completes the initialization if some checks are fullfiled. These checks are:
		///  -The reward contribution money matches the crowdloan pot
		///  -The end vesting block is higher than the init vesting block
		///  -The initialization has not complete yet
		#[pallet::weight(T::WeightInfo::complete_initialization())]
		pub fn complete_initialization(
			origin: OriginFor<T>,
			lease_ending_block: T::VestingBlockNumber,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let initialized = <Initialized<T>>::get();

			// This ensures there was no prior initialization
			ensure!(
				initialized == false,
				Error::<T>::RewardVecAlreadyInitialized
			);


			// This ensures the end vesting block (when all funds are fully vested)
			// is bigger than the init vesting block
			ensure!(
				lease_ending_block > InitVestingBlock::<T>::get(),
				Error::<T>::VestingPeriodNonValid
			);
			let current_initialized_rewards = InitializedRewardAmount::<T>::get();

			let reward_difference = Self::pot().saturating_sub(current_initialized_rewards);


			// Burn the difference
			let imbalance = T::RewardCurrency::withdraw(
				&PALLET_ID.into_account(),
				reward_difference,
				WithdrawReasons::TRANSFER,
				AllowDeath,
			)
			.expect("Shouldnt fail, as the fund should be enough to burn and nothing is locked");
			drop(imbalance);

			EndVestingBlock::<T>::put(lease_ending_block);

			<Initialized<T>>::put(true);

			Ok(Default::default())
		}

	}

	impl<T: Config> Pallet<T> {
		/// The account ID that holds the Crowdloan's funds
		pub fn account_id() -> T::AccountId {
			PALLET_ID.into_account()
		}
		/// The Account Id's balance
		pub fn pot() -> BalanceOf<T> {
			T::RewardCurrency::free_balance(&Self::account_id())
		}
		/// Verify a set of signatures made with relay chain accounts
		/// We are verifying all the signatures, and then counting
		/// We could do something more efficient like count as we verify
		/// In any of the cases the weight will need to account for all the signatures,
		/// as we dont know beforehand whether they will be valid
		fn verify_signatures(
			proofs: Vec<(T::RelayChainAccountId, MultiSignature)>,
			reward_info: RewardInfo<T>,
			_payload: Vec<u8>,
		) -> DispatchResult {
			// The proofs should
			// 1. be signed by contributors to this address, otherwise they are not counted
			// 2. Signs a valid native identity
			// 3. The sum of the valid proofs needs to be bigger than InsufficientNumberOfValidProofs

			// I use a map here for faster lookups
			let mut voted: BTreeMap<T::RelayChainAccountId, ()> = BTreeMap::new();
			for (relay_account, _signature) in proofs {
				// We just count votes that we have not seen
				if voted.get(&relay_account).is_none() {
					// Maybe I should not error here?
					ensure!(
						reward_info
							.contributed_relay_addresses
							.contains(&relay_account),
						Error::<T>::NonContributedAddressProvided
					);

					// // I am erroring here as I think it is good to know the reason in the single-case
					// // signature
					// ensure!(
					// 	signature.verify(payload.as_slice(), &relay_account.clone().into()),
					// 	Error::<T>::InvalidClaimSignature
					// );
					voted.insert(relay_account, ());
				}
			}

			// Ensure the votes are sufficient
			ensure!(
				Perbill::from_rational(
					voted.len() as u32,
					reward_info.contributed_relay_addresses.len() as u32
				) >= T::RewardAddressRelayVoteThreshold::get(),
				Error::<T>::InsufficientNumberOfValidProofs
			);
			Ok(())
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// User trying to associate a native identity with a relay chain identity for posterior
		/// reward claiming provided an already associated relay chain identity
		AlreadyAssociated,
		/// Trying to introduce a batch that goes beyond the limits of the funds
		BatchBeyondFundPot,
		/// First claim already done
		FirstClaimAlreadyDone,
		/// The contribution is not high enough to be eligible for rewards
		RewardNotHighEnough,
		/// User trying to associate a native identity with a relay chain identity for posterior
		/// reward claiming provided a wrong signature
		InvalidClaimSignature,
		/// User trying to claim the first free reward provided the wrong signature
		InvalidFreeClaimSignature,
		/// User trying to claim an award did not have an claim associated with it. This may mean
		/// they did not contribute to the crowdloan, or they have not yet associated a native id
		/// with their contribution
		NoAssociatedClaim,
		/// User trying to claim rewards has already claimed all rewards associated with its
		/// identity and contribution
		RewardsAlreadyClaimed,
		/// Reward vec has already been initialized
		RewardVecAlreadyInitialized,
		/// Reward vec has not yet been fully initialized
		RewardVecNotFullyInitializedYet,
		/// Rewards should match funds of the pallet
		RewardsDoNotMatchFund,
		/// Initialize_reward_vec received too many contributors
		TooManyContributors,
		/// Provided vesting period is not valid
		VestingPeriodNonValid,
		/// User provided a signature from a non-contributor relay account
		NonContributedAddressProvided,
		/// User submitted an unsifficient number of proofs to change the reward address
		InsufficientNumberOfValidProofs,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// The amount of funds this pallet controls
		pub funded_amount: BalanceOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				funded_amount: 1u32.into(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		// This sets the funds of the crowdloan pallet
		fn build(&self) {
			T::RewardCurrency::deposit_creating(&Pallet::<T>::account_id(), self.funded_amount);
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn accounts_payable)]
	pub type AccountsPayable<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, RewardInfo<T>>;
	#[pallet::storage]
	#[pallet::getter(fn claimed_relay_chain_ids)]
	pub type ClaimedRelayChainIds<T: Config> =
		StorageMap<_, Blake2_128Concat, T::RelayChainAccountId, ()>;
	#[pallet::storage]
	#[pallet::getter(fn unassociated_contributions)]
	pub type UnassociatedContributions<T: Config> =
		StorageMap<_, Blake2_128Concat, T::RelayChainAccountId, RewardInfo<T>>;
	#[pallet::storage]
	#[pallet::getter(fn initialized)]
	pub type Initialized<T: Config> = StorageValue<_, bool, ValueQuery, T::Initialized>;

	#[pallet::storage]
	#[pallet::storage_prefix = "InitRelayBlock"]
	#[pallet::getter(fn init_vesting_block)]
	/// Vesting block height at the initialization of the pallet
	type InitVestingBlock<T: Config> = StorageValue<_, T::VestingBlockNumber, ValueQuery>;

	#[pallet::storage]
	#[pallet::storage_prefix = "EndRelayBlock"]
	#[pallet::getter(fn end_vesting_block)]
	/// Vesting block height at the initialization of the pallet
	type EndVestingBlock<T: Config> = StorageValue<_, T::VestingBlockNumber, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn init_reward_amount)]
	/// Total initialized amount so far. We store this to make pallet funds == contributors reward
	/// check easier and more efficient
	type InitializedRewardAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_contributors)]
	/// Total number of contributors to aid hinting benchmarking
	type TotalContributors<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// The initial payment of InitializationPayment % was paid
		InitialPaymentMade(T::AccountId, BalanceOf<T>),
		/// Someone has proven they made a contribution and associated a native identity with it.
		/// Data is the relay account,  native account and the total amount of _rewards_ that will be paid
		NativeIdentityAssociated(T::RelayChainAccountId, T::AccountId, BalanceOf<T>),
		/// A contributor has claimed some rewards.
		/// Data is the account getting paid and the amount of rewards paid.
		RewardsPaid(T::AccountId, BalanceOf<T>),
		/// A contributor has updated the reward address.
		RewardAddressUpdated(T::AccountId, T::AccountId),
		/// When initializing the reward vec an already initialized account was found
		InitializedAlreadyInitializedAccount(
			T::RelayChainAccountId,
			Option<T::AccountId>,
			BalanceOf<T>,
		),
		/// When initializing the reward vec an already initialized account was found
		InitializedAccountWithNotEnoughContribution(
			T::RelayChainAccountId,
			Option<T::AccountId>,
			BalanceOf<T>,
		),
	}
}


pub type BalanceOf<T> = <<T as Config>::RewardCurrency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

/// Stores info about the rewards owed as well as how much has been vested so far.
/// For a primer on this kind of design, see the recipe on compounding interest
/// https://substrate.dev/recipes/fixed-point.html#continuously-compounding
#[derive(Clone, PartialEq, Eq, Encode, Decode, Default, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RewardInfo<T: Config> {
	pub total_reward: BalanceOf<T>,
	pub claimed_reward: BalanceOf<T>,
	pub contributed_relay_addresses: Vec<T::RelayChainAccountId>,
}