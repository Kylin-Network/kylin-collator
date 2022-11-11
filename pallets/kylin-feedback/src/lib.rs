#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::too_many_arguments)]

use frame_support::{
	dispatch::DispatchResult, ensure, traits::tokens::nonfungibles::*, BoundedVec,
};
use frame_system::{
	ensure_signed,
    Config as SystemConfig,
};
use sp_runtime::{traits::StaticLookup, DispatchError, Permill};
use sp_std::{prelude::*, str, vec::Vec};
use sp_std::convert::TryInto;
use sp_std::result::Result;
use cumulus_pallet_xcm::{ensure_sibling_para, Origin as CumulusOrigin};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);


	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeOrigin: From<<Self as SystemConfig>::RuntimeOrigin>
            + Into<Result<CumulusOrigin, <Self as Config>::RuntimeOrigin>>;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		QueryFeedBack {
			key: Vec<u8>,
			value: i64,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		TooLong,
		NoAvailableCollectionId,
		NoAvailableResourceId,
		MetadataNotSet,
		RecipientNotSet,
		NoAvailableNftId,
		NotInRange,
		CollectionUnknown,
		NoPermission,
		NoWitness,
		CollectionNotEmpty,
		CollectionFullOrLocked,
		CannotSendToDescendentOrSelf,
		ResourceAlreadyExists,
		EmptyResource,
		TooManyRecursions,
		NftIsLocked,
		CannotAcceptNonOwnedNft,
		CannotRejectNonOwnedNft,
		ResourceDoesntExist,
		/// Accepting a resource that is not pending should fail
		ResourceNotPending,
		NonTransferable,
	}


	#[pallet::call]
	impl<T: Config> Pallet<T>
	{
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn xcm_feed_back(
			origin: OriginFor<T>,
			key: Vec<u8>,
			value: i64,
		) -> DispatchResult {
			let para_id =
                ensure_sibling_para(<T as Config>::RuntimeOrigin::from(origin.clone()))?;

			Self::deposit_event(Event::QueryFeedBack { key, value });
			Ok(())
		}

	}
}
