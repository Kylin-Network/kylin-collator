#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod models;
pub mod weights;

#[cfg(any(feature = "runtime-benchmarks", test))]
mod benchmarking;
mod mocks;

#[frame_support::pallet]
pub mod pallet {
	use crate::{
		models::{Distribution, DistributionState, Identity, Proof, RecipientFund},
		weights::WeightInfo,
	};
	use codec::{Codec, FullCodec, MaxEncodedLen};
	use kylin_support::{
		abstractions::{
			nonce::Nonce,
			utils::{
				increment::{Increment, SafeIncrement},
				start_at::ZeroInit,
			},
		},
		math::safe::{SafeAdd, SafeSub},
		signature_verification,
	};
	
	/// Contains functions necessary for the business logic for managing Distributions
pub trait Distributionper {
	type AccountId;
	type DistributionId;
	type DistributionStart;
	type Balance;
	type Proof;
	type Recipient;
	type RecipientCollection;
	type Identity;
	type VestingSchedule;

	/// Create a new Distribution.
	fn create_distribution(
		creator_id: Self::AccountId,
		start: Option<Self::DistributionStart>,
		schedule: Self::VestingSchedule,
	) -> DispatchResult;

	/// Add one or more recipients to an Distribution.
	fn add_recipient(
		origin_id: Self::AccountId,
		distribution_id: Self::DistributionId,
		recipients: Self::RecipientCollection,
	) -> DispatchResult;

	/// Remove a recipient from an Distribution.
	fn remove_recipient(
		origin_id: Self::AccountId,
		distribution_id: Self::DistributionId,
		recipient: Self::Recipient,
	) -> DispatchResult;

	/// Start an Distribution.
	fn enable_distribution(origin_id: Self::AccountId, distribution_id: Self::DistributionId) -> DispatchResult;

	/// Stop an Distribution.
	fn disable_distribution(
		origin_id: Self::AccountId,
		distribution_id: Self::DistributionId,
	) -> Result<Self::Balance, DispatchError>;

	/// Claim a recipient reward from an Distribution.
	fn claim(
		distribution_id: Self::DistributionId,
		remote_account: Self::Identity,
		reward_account: Self::AccountId,
	) -> DispatchResultWithPostInfo;
}

	use frame_support::{
		dispatch::PostDispatchInfo,
		pallet_prelude::*,
		traits::{
			fungible::{Inspect, Transfer},
			Time,
		},
		transactional, Blake2_128Concat, PalletId, Parameter,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_runtime::{
		traits::{
			AccountIdConversion, AtLeast32Bit, AtLeast32BitUnsigned, CheckedAdd, CheckedMul,
			CheckedSub, Convert, One, Saturating, Zero,
		},
		AccountId32, DispatchErrorWithPostInfo,
	};
	use sp_std::{fmt::Debug, vec::Vec};

	/// [`AccountId`](frame_system::Config::AccountId) as configured by the pallet.
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	/// [`DistributionId`](Config::DistributionId) as configured by the pallet.
	pub type DistributionIdOf<T> = <T as Config>::DistributionId;
	/// [`Distribution`](crate::models::Distribution) as configured by the pallet.
	pub type DistributionOf<T> = Distribution<
		<T as frame_system::Config>::AccountId,
		<T as Config>::Balance,
		<T as Config>::Moment,
	>;
	/// [`Balance`](Config::Balance) as configured by the pallet.
	pub type BalanceOf<T> = <T as Config>::Balance;
	/// [`RecipientFund`](crate::models::RecipientFund) as configured by the pallet.
	pub type RecipientFundOf<T> = RecipientFund<<T as Config>::Balance, <T as Config>::Moment>;
	/// [`Moment`](Config::Moment) as configured by the pallet.
	pub type MomentOf<T> = <T as Config>::Moment;
	/// ['Proof'](crate::models::Proof) as configured by the pallet
	pub type ProofOf<T> = Proof<<T as Config>::RelayChainAccountId>;
	pub type IdentityOf<T> = Identity<<T as Config>::RelayChainAccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DistributionCreated {
			distribution_id: T::DistributionId,
			by: T::AccountId,
		},
		RecipientsAdded {
			distribution_id: T::DistributionId,
			number: u32,
			unclaimed_funds: T::Balance,
		},
		RecipientRemoved {
			distribution_id: T::DistributionId,
			recipient_id: IdentityOf<T>,
			unclaimed_funds: T::Balance,
		},
		DistributionStarted {
			distribution_id: T::DistributionId,
			at: T::Moment,
		},
		DistributionEnded {
			distribution_id: T::DistributionId,
			at: T::Moment,
		},
		Claimed {
			identity: IdentityOf<T>,
			recipient_account: T::AccountId,
			amount: T::Balance,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		DistributionAlreadyStarted,
		DistributionDoesNotExist,
		DistributionIsNotEnabled,
		ArithmiticError,
		AssociatedWithAnohterAccount,
		BackToTheFuture,
		NotDistributionCreator,
		NothingToClaim,
		RecipientAlreadyClaimed,
		RecipientNotFound,
		InvalidProof,
		UnclaimedFundsRemaining,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Distribution ID
		type DistributionId: Copy
			+ Clone
			+ Eq
			+ Debug
			+ Zero
			+ One
			+ SafeAdd
			+ FullCodec
			+ MaxEncodedLen
			+ Parameter
			+ TypeInfo;

		/// Representation of some amount of tokens
		type Balance: Default
			+ Parameter
			+ Codec
			+ Copy
			+ Ord
			+ CheckedAdd
			+ CheckedSub
			+ CheckedMul
			+ AtLeast32BitUnsigned
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ Zero;

		/// Conversion function from [`Self::Moment`] to [`Self::Balance`]
		type Convert: Convert<Self::Moment, Self::Balance>;

		/// Time stamp
		type Moment: AtLeast32Bit + Parameter + Default + Copy + MaxEncodedLen + FullCodec;

		/// Relay chain account ID
		type RelayChainAccountId: Parameter
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ Into<AccountId32>
			+ Ord;

		/// The asset type Recipients will claim from the Distributions.
		type RecipientFundAsset: Inspect<Self::AccountId, Balance = Self::Balance>
			+ Transfer<Self::AccountId, Balance = Self::Balance>;

		/// Time provider
		type Time: Time<Moment = Self::Moment>;

		/// The pallet ID required for creating sub-accounts used by Distributions.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The prefix used in proofs
		#[pallet::constant]
		type Prefix: Get<&'static [u8]>;

		/// The stake required to craete an Distribution
		#[pallet::constant]
		type Stake: Get<BalanceOf<Self>>;

		/// The implementation of extrinsic weights.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The counter used to identify Distributions.
	#[pallet::storage]
	#[pallet::getter(fn distribution_count)]
	#[allow(clippy::disallowed_types)] // Allow `frame_support::pallet_prelude::ValueQuery` because default of 0 is correct
	pub type DistributionCount<T: Config> =
		StorageValue<_, T::DistributionId, ValueQuery, Nonce<ZeroInit, SafeIncrement>>;

	/// Distributions currently stored by the pallet.
	#[pallet::storage]
	#[pallet::getter(fn distributions)]
	pub type Distributions<T: Config> =
		StorageMap<_, Blake2_128Concat, T::DistributionId, DistributionOf<T>, OptionQuery>;

	/// Associations of local accounts and an [`DistributionId`](Config::DistributionId) to a remote account.
	#[pallet::storage]
	#[pallet::getter(fn associations)]
	pub type Associations<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::DistributionId,
		Blake2_128Concat,
		T::AccountId,
		IdentityOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn total_distribution_recipients)]
	#[allow(clippy::disallowed_types)] // Allow `frame_support::pallet_prelude::ValueQuery` because default of 0 is correct
	pub type TotalDistributionRecipients<T: Config> =
		StorageMap<_, Blake2_128Concat, T::DistributionId, u32, ValueQuery>;

	/// Recipient funds of Distributions stored by the pallet.
	#[pallet::storage]
	#[pallet::getter(fn recipient_funds)]
	pub type RecipientFunds<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::DistributionId,
		Blake2_128Concat,
		IdentityOf<T>,
		RecipientFundOf<T>,
		OptionQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new Distribution. This requires that the user puts down a stake in PICA.
		///
		/// If `start_at` is `Some(MomentOf<T>)` and the `MomentOf<T>` is greater than the current
		/// block, the Distribution will be scheduled to start automatically.
		///
		/// Can be called by any signed origin.
		///
		/// # Parameter Sources
		/// * `start_at` - user provided, optional
		/// * `vesting_schedule` - user provided
		///
		/// # Emits
		/// * `DistributionCreated`
		/// * `DistributionStarted`
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `DistributionAlreadyStarted` - The Distribution has already started or has been scheduled to
		/// start
		/// * `BackToTheFuture` - The provided `start` has already passed
		#[pallet::weight(<T as Config>::WeightInfo::create_distribution())]
		#[transactional]
		pub fn create_distribution(
			origin: OriginFor<T>,
			start_at: Option<MomentOf<T>>,
			vesting_schedule: MomentOf<T>,
		) -> DispatchResult {
			let creator = ensure_signed(origin)?;

			<Self as Distributionper>::create_distribution(creator, start_at, vesting_schedule)
		}

		/// Add one or more recipients to the Distribution, specifying the token amount that each
		/// provided address will receive.
		///
		/// Only callable by the origin that created the Distribution.
		///
		/// # Parameter Sources
		/// * `distribution_id` - user selected, provided by the system
		/// * `recipients` - user provided
		///
		/// # Emits
		/// * `RecipientsAdded`
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `NotDistributionCreator` - Signer of the origin is not the creator of the Distribution
		#[pallet::weight(<T as Config>::WeightInfo::add_recipient(recipients.len() as u32))]
		#[transactional]
		pub fn add_recipient(
			origin: OriginFor<T>,
			distribution_id: T::DistributionId,
			recipients: Vec<(IdentityOf<T>, BalanceOf<T>, MomentOf<T>, bool)>,
		) -> DispatchResult {
			let origin_id = ensure_signed(origin)?;

			<Self as Distributionper>::add_recipient(origin_id, distribution_id, recipients)
		}

		/// Remove a recipient from an Distribution.
		///
		/// Only callable by the origin that created the Distribution.
		///
		/// # Parameter Sources
		/// * `distribution_id` - user selected, provided by the system
		/// * `recipient` - user selected, provided by the system
		///
		/// # Emits
		/// * `RecipientRemoved`
		/// * `DistributionEnded`
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `NotDistributionCreator` - Signer of the origin is not the creator of the Distribution
		/// * `RecipientAlreadyClaimed` - The recipient has already began claiming their funds.
		/// * `RecipientNotFound` - No recipient associated with the `identity` could be found.
		#[pallet::weight(<T as Config>::WeightInfo::remove_recipient())]
		#[transactional]
		pub fn remove_recipient(
			origin: OriginFor<T>,
			distribution_id: T::DistributionId,
			recipient: IdentityOf<T>,
		) -> DispatchResult {
			let origin_id = ensure_signed(origin)?;

			<Self as Distributionper>::remove_recipient(origin_id, distribution_id, recipient)
		}

		/// Start an Distribution.
		///
		/// Only callable by the origin that created the Distribution.
		///
		/// # Parameter Sources
		/// * `distribution_id` - user selected, provided by the system
		///
		/// # Emits
		/// * `DistributionStarted`
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `DistributionAlreadyStarted` - The Distribution has already started or has been scheduled to
		/// start
		/// * `BackToTheFuture` - The provided `start` has already passed
		/// * `NotDistributionCreator` - Signer of the origin is not the creator of the Distribution
		#[pallet::weight(<T as Config>::WeightInfo::enable_distribution())]
		#[transactional]
		pub fn enable_distribution(origin: OriginFor<T>, distribution_id: T::DistributionId) -> DispatchResult {
			let origin_id = ensure_signed(origin)?;

			<Self as Distributionper>::enable_distribution(origin_id, distribution_id)
		}

		/// Stop an Distribution.
		///
		/// Only callable by the origin that created the Distribution.
		///
		/// # Parameter Sources
		/// * `distribution_id` - user selected, provided by the system
		///
		/// # Emits
		/// * `DistributionEnded`
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `NotDistributionCreator` - Signer of the origin is not the creator of the Distribution
		#[pallet::weight(<T as Config>::WeightInfo::disable_distribution())]
		#[transactional]
		pub fn disable_distribution(origin: OriginFor<T>, distribution_id: T::DistributionId) -> DispatchResult {
			let origin_id = ensure_signed(origin)?;

			<Self as Distributionper>::disable_distribution(origin_id, distribution_id)?;
			Ok(())
		}

		/// Claim recipient funds from an Distribution.
		///
		/// If no more funds are left to claim, the Distribution will be removed.
		///
		/// Callable by any unsigned origin.
		///
		/// # Parameter Sources
		/// * `distribution_id` - user selected, provided by the system
		/// * `reward_account` - user provided
		/// * `proof` - calculated by the system (requires applicable signing)
		///
		/// # Emits
		/// * `DistributionEnded`
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `DistributionIsNotEnabled` - The Distribution has not been enabled
		/// * `AssociatedWithAnohterAccount` - Associated with a different account
		/// * `ArithmiticError` - Overflow while totaling claimed funds
		/// * `InvalidProof`
		/// * `RecipientNotFound` - No recipient associated with the `identity` could be found.
		#[pallet::weight(<T as Config>::WeightInfo::claim(TotalDistributionRecipients::<T>::get(distribution_id)))]
		#[transactional]
		pub fn claim(
			origin: OriginFor<T>,
			distribution_id: T::DistributionId,
			reward_account: T::AccountId,
			proof: ProofOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;
			let identity = Self::get_identity(proof, &reward_account, T::Prefix::get())?;

			match Associations::<T>::get(distribution_id, reward_account.clone()) {
				// Confirm association matches
				Some(associated_account) => {
					ensure!(
						associated_account == identity,
						Error::<T>::AssociatedWithAnohterAccount
					);
				},
				// If no association exists, create a new one
				None => {
					Associations::<T>::insert(distribution_id, reward_account.clone(), identity.clone());
				},
			}

			<Self as Distributionper>::claim(distribution_id, identity, reward_account)
		}
	}

	#[pallet::extra_constants]
	impl<T: Config> Pallet<T> {
		/// The AccountId of this pallet.
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}

	impl<T: Config> Pallet<T> {
		/// Gets the account ID to be used by the Distribution.
		pub(crate) fn get_distribution_account_id(distribution_id: T::DistributionId) -> AccountIdOf<T> {
			T::PalletId::get().into_sub_account_truncating(distribution_id)
		}

		/// Gets the [`Distribution`](crate::models::Distribution) associated with the `distribution_id`
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		pub(crate) fn get_distribution(distribution_id: &T::DistributionId) -> Result<DistributionOf<T>, Error<T>> {
			Distributions::<T>::try_get(distribution_id).map_err(|_| Error::<T>::DistributionDoesNotExist)
		}

		/// Calculates the current [`DistributionState`](crate::models::DistributionState) of an Distribution
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		pub(crate) fn get_distribution_state(
			distribution_id: T::DistributionId,
		) -> Result<DistributionState, Error<T>> {
			let distribution = Self::get_distribution(&distribution_id)?;

			if distribution.disabled {
				return Ok(DistributionState::Disabled)
			}

			distribution.start.map_or(Ok(DistributionState::Created), |start| {
				if start <= T::Time::now() {
					Ok(DistributionState::Enabled)
				} else {
					Ok(DistributionState::Created)
				}
			})
		}

		/// Gets the [`RecipientFund`](crate::models::RecipientFund) of an Distribution that is
		/// associated with the `identity`.
		///
		/// # Errors
		/// * `RecipientNotFound` - No recipient associated with the `identity` could be found.
		pub(crate) fn get_recipient_fund(
			distribution_id: T::DistributionId,
			identity: IdentityOf<T>,
		) -> Result<RecipientFundOf<T>, Error<T>> {
			RecipientFunds::<T>::try_get(distribution_id, identity)
				.map_err(|_| Error::<T>::RecipientNotFound)
		}

		/// Gets the remote account address from the `Proof`.
		///
		/// # Errors
		/// * `InvalidProof` - If the proof is invalid, an error will be returned.
		pub(crate) fn get_identity(
			proof: ProofOf<T>,
			reward_account: &<T as frame_system::Config>::AccountId,
			prefix: &[u8],
		) -> Result<IdentityOf<T>, DispatchErrorWithPostInfo<PostDispatchInfo>> {
			let identity = match proof {
				Proof::Ethereum(eth_proof) => {
					let reward_account_encoded =
						reward_account.using_encoded(signature_verification::get_encoded_vec);
					let eth_address = signature_verification::ethereum_recover(
						prefix,
						&reward_account_encoded,
						&eth_proof,
					)
					.map_err(|_| Error::<T>::InvalidProof)?;
					Result::<_, DispatchError>::Ok(Identity::Ethereum(eth_address))
				},
				Proof::RelayChain(relay_account, relay_proof) => {
					ensure!(
						signature_verification::verify_relay(
							prefix,
							reward_account.clone(),
							relay_account.clone().into(),
							&relay_proof
						),
						Error::<T>::InvalidProof
					);
					Ok(Identity::RelayChain(relay_account))
				},
			}?;
			Ok(identity)
		}

		/// Start an Distribution at a given moment.
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `DistributionAlreadyStarted` - The Distribution has already started or has been scheduled to
		/// start
		/// * `BackToTheFuture` - The provided `start` has already passed
		pub(crate) fn start_distribution_at(
			distribution_id: T::DistributionId,
			start: T::Moment,
		) -> DispatchResult {
			// Start is valid
			let now = T::Time::now();
			ensure!(start >= now, Error::<T>::BackToTheFuture);
			// Distribution exist and hasn't started
			let distribution = Self::get_distribution(&distribution_id)?;
			ensure!(distribution.start.is_none(), Error::<T>::DistributionAlreadyStarted);

			// Update Distribution
			Distributions::<T>::try_mutate(distribution_id, |distribution| match distribution.as_mut() {
				Some(distribution) => {
					distribution.start = Some(start);
					Ok(())
				},
				None => Err(Error::<T>::DistributionDoesNotExist),
			})?;

			Self::deposit_event(Event::DistributionStarted { distribution_id, at: start });

			Ok(())
		}

		/// Calculates the amount of the total fund that a recipient should have claimed.
		///
		/// The amount that should have been claimed is proportional to the number of **full**
		/// vesting steps passed.
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `DistributionIsNotEnabled` - The Distribution has not been enabled
		pub(crate) fn claimable(
			distribution_id: T::DistributionId,
			fund: &RecipientFundOf<T>,
		) -> Result<T::Balance, Error<T>> {
			let distribution = Distributions::<T>::get(distribution_id).ok_or(Error::<T>::DistributionDoesNotExist)?;
			let distribution_state = Self::get_distribution_state(distribution_id)?;
			match (distribution_state, distribution.start) {
				(DistributionState::Enabled, Some(start)) => {
					let now = T::Time::now();
					let vesting_point = now.saturating_sub(start);

					// If the vesting period is over, the recipient should receive the remainder of
					// the fund
					if vesting_point >= fund.vesting_period {
						return Ok(fund.total)
					}

					// The current vesting window rounded to the previous window
					let vesting_window =
						vesting_point.saturating_sub(vesting_point % distribution.schedule);

					let claimable = fund.total.saturating_mul(T::Convert::convert(vesting_window)) /
						T::Convert::convert(fund.vesting_period);

					Ok(claimable)
				},
				_ => Err(Error::<T>::DistributionIsNotEnabled),
			}
		}

		/// Removes an Distribution and associated data from the pallet iff all funds have been recorded
		/// as claimed.
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		pub(crate) fn prune_distribution(distribution_id: T::DistributionId) -> Result<bool, DispatchError> {
			let distribution = Self::get_distribution(&distribution_id)?;
			let distribution_account = Self::get_distribution_account_id(distribution_id);

			if distribution.total_funds > distribution.claimed_funds {
				return Ok(false)
			}

			// Return remaining funds to the Distribution creator
			T::RecipientFundAsset::transfer(
				&distribution_account,
				&distribution.creator,
				T::RecipientFundAsset::balance(&distribution_account),
				false,
			)?;

			// Remove Distribution and associated data from storage

			// NOTE(hussein-aitlahcen): this is deprecated, but the new API state in the doc that we
			// can have an infinite limit. while the new `clear_prefix` signature doesn't match this
			// definition (force u32 as limit). Missing feature or limit is forced? Who know.
			#[allow(deprecated)]
			RecipientFunds::<T>::remove_prefix(distribution_id, None);
			#[allow(deprecated)]
			Associations::<T>::remove_prefix(distribution_id, None);
			Distributions::<T>::remove(distribution_id);

			Ok(true)
		}
	}

	impl<T: Config> Distributionper for Pallet<T> {
		type AccountId = AccountIdOf<T>;
		type DistributionId = DistributionIdOf<T>;
		type DistributionStart = MomentOf<T>;
		type Balance = BalanceOf<T>;
		type Proof = ProofOf<T>;
		type Recipient = IdentityOf<T>;
		type RecipientCollection = Vec<(Self::Recipient, BalanceOf<T>, MomentOf<T>, bool)>;
		type Identity = IdentityOf<T>;
		type VestingSchedule = MomentOf<T>;

		/// Create a new Distribution.
		///
		/// Provide `None` for `start` if starting the Distribution manually is desired.
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `DistributionAlreadyStarted` - The Distribution has already started or has been scheduled to
		/// start
		/// * `BackToTheFuture` - The provided `start` has already passed
		fn create_distribution(
			creator_id: Self::AccountId,
			start: Option<Self::DistributionStart>,
			schedule: Self::VestingSchedule,
		) -> DispatchResult {
			let distribution_id = DistributionCount::<T>::increment()?;
			let distribution_account = Self::get_distribution_account_id(distribution_id);

			// Insert newly created distribution into pallet's list.
			Distributions::<T>::insert(
				distribution_id,
				Distribution {
					creator: creator_id.clone(),
					total_funds: T::Balance::zero(),
					total_recipients: 0,
					claimed_funds: T::Balance::zero(),
					start: None,
					schedule,
					disabled: false,
				},
			);

			// Transfer stake into distribution specific account.
			T::RecipientFundAsset::transfer(&creator_id, &distribution_account, T::Stake::get(), false)?;

			Self::deposit_event(Event::DistributionCreated { distribution_id, by: creator_id });

			if let Some(moment) = start {
				Self::start_distribution_at(distribution_id, moment)?;
			}

			Ok(())
		}

		/// Add one or more recipients to an Distribution.
		///
		/// Distribution creator is expected to be able to fund the Distribution. If the Distributions current
		/// funds aren't enough to supply all claims, the creator will be charged the difference.
		///
		/// If a recipient is already a member of an Distribution, their previous entry will be
		/// replaced for that Distribution.
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `NotDistributionCreator` - Signer of the origin is not the creator of the Distribution
		fn add_recipient(
			origin_id: Self::AccountId,
			distribution_id: Self::DistributionId,
			recipients: Self::RecipientCollection,
		) -> DispatchResult {
			let distribution = Self::get_distribution(&distribution_id)?;
			ensure!(distribution.creator == origin_id, Error::<T>::NotDistributionCreator);

			// Calculate total funds and recipients local to this transaction
			let (transaction_funds, transaction_recipients) = recipients.iter().try_fold(
				(T::Balance::zero(), 0),
				|(transaction_funds, transaction_recipients),
				 (_, funds, _, _)|
				 -> Result<(T::Balance, u32), DispatchError> {
					Ok((transaction_funds.safe_add(funds)?, transaction_recipients.safe_add(&1)?))
				},
			)?;

			// Funds currently owned by the Distribution minus the creation stake
			let current_funds =
				T::RecipientFundAsset::balance(&Self::get_distribution_account_id(distribution_id))
					.safe_sub(&T::Stake::get())?;
			// Total amount of funds to be required by this Distribution
			let total_funds = distribution.total_funds.safe_add(&transaction_funds)?;
			let total_recipients = distribution.total_recipients.safe_add(&transaction_recipients)?;

			// If the distribution can't support the total amount of claimable funds
			if current_funds < total_funds {
				// Fund Distribution account from creators account
				T::RecipientFundAsset::transfer(
					&distribution.creator,
					&Self::get_distribution_account_id(distribution_id),
					total_funds.safe_sub(&current_funds)?,
					false,
				)?;
			}

			// Populate `RecipientFunds`
			recipients.iter().for_each(|(identity, funds, vesting_period, is_funded)| {
				RecipientFunds::<T>::insert(
					distribution_id,
					identity,
					RecipientFundOf::<T> {
						total: *funds,
						claimed: T::Balance::zero(),
						vesting_period: *vesting_period,
						funded_claim: *is_funded,
					},
				);
			});

			TotalDistributionRecipients::<T>::mutate(distribution_id, |total_distribution_recipients| {
				*total_distribution_recipients = total_recipients;
			});

			// Update Distribution statistics
			let (total_funds, claimed_funds) =
				Distributions::<T>::try_mutate(distribution_id, |distribution| match distribution.as_mut() {
					Some(distribution) => {
						distribution.total_funds = total_funds;
						distribution.total_recipients = total_recipients;
						// Ok(distribution.total_funds.safe_sub(&distribution.claimed_funds)?)
						Ok((distribution.total_funds, distribution.claimed_funds))
					},
					None => Err(Error::<T>::DistributionDoesNotExist),
				})?;

			Self::deposit_event(Event::RecipientsAdded {
				distribution_id,
				number: transaction_recipients,
				unclaimed_funds: total_funds.safe_sub(&claimed_funds)?,
			});

			Ok(())
		}

		/// Remove a recipient from an Distribution.
		///
		/// Refunds the creator for the value of the recipient fund.
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `NotDistributionCreator` - Signer of the origin is not the creator of the Distribution
		/// * `RecipientAlreadyClaimed` - The recipient has already began claiming their funds.
		/// * `RecipientNotFound` - No recipient associated with the `identity` could be found.
		fn remove_recipient(
			origin_id: Self::AccountId,
			distribution_id: Self::DistributionId,
			recipient: Self::Recipient,
		) -> DispatchResult {
			let distribution = Self::get_distribution(&distribution_id)?;
			ensure!(distribution.creator == origin_id, Error::<T>::NotDistributionCreator);

			let distribution_account = Self::get_distribution_account_id(distribution_id);
			let recipient_fund = Self::get_recipient_fund(distribution_id, recipient.clone())?;

			ensure!(
				recipient_fund.claimed == T::Balance::zero(),
				Error::<T>::RecipientAlreadyClaimed
			);

			// Update Distribution details
			let (creator, total_funds, claimed_funds) =
				Distributions::<T>::try_mutate(distribution_id, |distribution| match distribution.as_mut() {
					Some(distribution) => {
						distribution.total_funds =
							distribution.total_funds.saturating_sub(recipient_fund.total);
						Ok((distribution.creator.clone(), distribution.total_funds, distribution.claimed_funds))
					},
					None => Err(Error::<T>::DistributionDoesNotExist),
				})?;

			TotalDistributionRecipients::<T>::mutate(distribution_id, |total_distribution_recipients| {
				*total_distribution_recipients -= 1;
			});

			// Refund Distribution creator for the recipient fund's value
			T::RecipientFundAsset::transfer(
				&distribution_account,
				&creator,
				recipient_fund.total,
				false,
			)?;

			RecipientFunds::<T>::remove(distribution_id, recipient.clone());

			Self::deposit_event(Event::RecipientRemoved {
				distribution_id,
				recipient_id: recipient,
				unclaimed_funds: total_funds.safe_sub(&claimed_funds)?,
			});

			if Self::prune_distribution(distribution_id)? {
				Self::deposit_event(Event::DistributionEnded { distribution_id, at: T::Time::now() })
			}

			Ok(())
		}

		/// Start an Distribution.
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `DistributionAlreadyStarted` - The Distribution has already started or has been scheduled to
		/// start
		/// * `BackToTheFuture` - The provided `start` has already passed
		/// * `NotDistributionCreator` - Signer of the origin is not the creator of the Distribution
		fn enable_distribution(
			origin_id: Self::AccountId,
			distribution_id: Self::DistributionId,
		) -> DispatchResult {
			let distribution = Self::get_distribution(&distribution_id)?;
			ensure!(distribution.creator == origin_id, Error::<T>::NotDistributionCreator);

			Self::start_distribution_at(distribution_id, T::Time::now())?;
			Ok(())
		}

		/// Stop an Distribution.
		///
		/// Returns the amount of unclaimed funds from the distribution upon success.
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `NotDistributionCreator` - Signer of the origin is not the creator of the Distribution
		fn disable_distribution(
			origin_id: Self::AccountId,
			distribution_id: Self::DistributionId,
		) -> Result<Self::Balance, DispatchError> {
			let distribution = Self::get_distribution(&distribution_id)?;
			ensure!(distribution.creator == origin_id, Error::<T>::NotDistributionCreator);

			let unclaimed_funds = Distributions::<T>::try_mutate(distribution_id, |distribution| {
				match distribution.as_mut() {
					Some(distribution) => {
						let at = T::Time::now();
						let unclaimed_funds = distribution.total_funds - distribution.claimed_funds;

						// REVIEW: Checking each recipient fund to see if they have started
						// claiming could prove to be expensive. Should we instead require that all
						// funds be claimed for an distribution to end?
						// sets claimed funds equal to total funds so the distribution can be pruned
						distribution.disabled = true;
						distribution.claimed_funds = distribution.total_funds;

						Self::deposit_event(Event::DistributionEnded { distribution_id, at });

						Ok(unclaimed_funds)
					},
					None => Err(Error::<T>::DistributionDoesNotExist.into()),
				}
			});

			Self::prune_distribution(distribution_id)?;

			unclaimed_funds
		}

		/// Claim a recipient reward from an Distribution.
		///
		/// # Errors
		/// * `DistributionDoesNotExist` - No Distribution exist that is associated 'distribution_id'
		/// * `DistributionIsNotEnabled` - The Distribution has not been enabled
		/// * `ArithmiticError` - Overflow while totaling claimed funds
		/// * `RecipientNotFound` - No recipient associated with the `identity` could be found.
		fn claim(
			distribution_id: Self::DistributionId,
			identity: Self::Identity,
			reward_account: Self::AccountId,
		) -> DispatchResultWithPostInfo {
			let distribution_account = Self::get_distribution_account_id(distribution_id);
			let (available_to_claim, recipient_fund) =
				RecipientFunds::<T>::try_mutate(distribution_id, identity, |fund| {
					match fund.as_mut() {
						Some(fund) => {
							let claimable = Self::claimable(distribution_id, fund)?;
							let available_to_claim = claimable.saturating_sub(fund.claimed);

							ensure!(
								available_to_claim > T::Balance::zero(),
								Error::<T>::NothingToClaim
							);

							// Update Distribution and fund status
							fund.claimed = fund.claimed.saturating_add(available_to_claim);

							Ok((available_to_claim, *fund))
						},
						None => Err(Error::<T>::RecipientNotFound),
					}
				})?;

			T::RecipientFundAsset::transfer(
				&distribution_account,
				&reward_account,
				available_to_claim,
				false,
			)?;

			Distributions::<T>::try_mutate(distribution_id, |distribution| match distribution.as_mut() {
				Some(distribution) => {
					distribution.claimed_funds = distribution
						.claimed_funds
						.safe_add(&available_to_claim)
						.map_err(|_| Error::<T>::ArithmiticError)?;
					Ok(())
				},
				None => Err(Error::<T>::DistributionDoesNotExist),
			})?;

			if Self::prune_distribution(distribution_id)? {
				Self::deposit_event(Event::DistributionEnded { distribution_id, at: T::Time::now() })
			}

			if recipient_fund.funded_claim {
				return Ok(Pays::No.into())
			}

			Ok(Pays::Yes.into())
		}
	}

	/// Ensures the following:
	/// * Only claim can be called via an unsigned transaction
	/// * The Distribution exists in the pallet's storage
	/// * The Distribution has been enabled / has started
	/// * The provided proof is valid
	/// * If an association has been created for the reward account, it matches the remote account
	/// * The recipient has funds to claim
	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(_: TransactionSource, call: &Self::Call) -> TransactionValidity {
			if let Call::claim { distribution_id, reward_account, proof } = call {
				// Validity Error if the distribution does not exist
				let distribution_state = Self::get_distribution_state(*distribution_id).map_err(|_| {
					Into::<TransactionValidityError>::into(InvalidTransaction::Custom(
						ValidityError::NotAnDistribution as u8,
					))
				})?;

				// Validity Error if the distribution has not started
				if distribution_state != DistributionState::Enabled {
					return InvalidTransaction::Custom(ValidityError::NotClaimable as u8).into()
				}

				// Evaluate proof
				let identity = Self::get_identity(proof.clone(), reward_account, T::Prefix::get())
					.map_err(|_| {
						Into::<TransactionValidityError>::into(InvalidTransaction::Custom(
							ValidityError::InvalidProof as u8,
						))
					})?;

				if let Some(associated_account) = Associations::<T>::get(distribution_id, reward_account)
				{
					// Validity Error if the account is already associated to another
					if associated_account != identity {
						return InvalidTransaction::Custom(ValidityError::AlreadyAssociated as u8)
							.into()
					}
				}

				// Validity Error if there are no funds for this recipient
				match RecipientFunds::<T>::get(distribution_id, identity.clone()) {
					None => InvalidTransaction::Custom(ValidityError::NoFunds as u8).into(),
					Some(fund) if fund.total.is_zero() =>
						InvalidTransaction::Custom(ValidityError::NoFunds as u8).into(),
					Some(_) => ValidTransaction::with_tag_prefix("DistributionAssociationCheck")
						.and_provides(identity)
						.build(),
				}
			} else {
				// Only allow unsigned transactions for `claim`
				Err(InvalidTransaction::Call.into())
			}
		}
	}

	pub enum ValidityError {
		InvalidProof,
		AlreadyAssociated,
		NoFunds,
		NotClaimable,
		NotAnDistribution,
	}
}
