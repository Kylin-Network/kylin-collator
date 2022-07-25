#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::too_many_arguments)]

use frame_support::{
	dispatch::DispatchResult, ensure, traits::tokens::nonfungibles::*, transactional, BoundedVec,
};
use frame_system::ensure_signed;

use sp_runtime::{traits::StaticLookup, DispatchError, Permill};
use kylin_primitives::nft::NftInfo;
use kylin_primitives::priority::Priority;
use kylin_primitives::property::Property;
use kylin_primitives::nft::{Nft, AccountIdOrCollectionNftTuple};
use kylin_primitives::resource::{Resource, ResourceInfo, ResourceTypes};
use kylin_primitives::collection::{Collection, CollectionInfo};
use kylin_primitives::types::*;
use sp_std::convert::TryInto;

use sp_std::result::Result;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod func;
pub use pallet::*;

pub type InstanceInfoOf<T> = NftInfo<
	<T as frame_system::Config>::AccountId,
	BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>,
>;
pub type ResourceOf<T, P> = ResourceInfo<BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>, BoundedVec<PartId, P>>;

pub type BoundedCollectionSymbolOf<T> = BoundedVec<u8, <T as Config>::CollectionSymbolLimit>;

pub type StringLimitOf<T> = BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>;

pub type BoundedResource<R> = BoundedVec<u8, R>;

pub type KeyLimitOf<T> = BoundedVec<u8, <T as pallet_uniques::Config>::KeyLimit>;

pub type ValueLimitOf<T> = BoundedVec<u8, <T as pallet_uniques::Config>::ValueLimit>;

pub type BoundedResourceTypeOf<T> = BoundedVec<
	ResourceTypes<
		BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>,
		BoundedVec<PartId, <T as Config>::PartsLimit>,
	>,
	<T as Config>::MaxResourcesOnMint,
>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use kylin_primitives::collection::CollectionInfo;
	use kylin_primitives::nft::AccountIdOrCollectionNftTuple;
	use kylin_primitives::resource::{BasicResource, ComposableResource, SlotResource};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_nft_id)]
	pub type NextNftId<T: Config> = StorageMap<_,
		Twox64Concat, CollectionId,
		NftId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn collection_index)]
	pub type CollectionIndex<T: Config> = StorageValue<_, CollectionId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_resource_id)]
	pub type NextResourceId<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat, CollectionId,
		Twox64Concat, NftId,
		ResourceId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn collections)]
	pub type Collections<T: Config> = StorageMap<
		_,
		Twox64Concat, CollectionId,
		CollectionInfo<StringLimitOf<T>, BoundedCollectionSymbolOf<T>, T::AccountId>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn nfts)]
	pub type Nfts<T: Config> =
	StorageDoubleMap<_,
		Twox64Concat, CollectionId,
		Twox64Concat, NftId,
		InstanceInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn priorities)]
	pub type Priorities<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, CollectionId>,
			NMapKey<Blake2_128Concat, NftId>,
			NMapKey<Blake2_128Concat, ResourceId>,
		),
		u32,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn children)]
	pub type Children<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat, (CollectionId, NftId),
		Twox64Concat, (CollectionId, NftId),
		(),
	>;

	#[pallet::storage]
	#[pallet::getter(fn resources)]
	pub type Resources<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, CollectionId>,
			NMapKey<Blake2_128Concat, NftId>,
			NMapKey<Blake2_128Concat, ResourceId>,
		),
		ResourceOf<T, T::PartsLimit>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn properties)]
	pub(super) type Properties<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, CollectionId>,
			NMapKey<Blake2_128Concat, Option<NftId>>,
			NMapKey<Blake2_128Concat, KeyLimitOf<T>>,
		),
		ValueLimitOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn lock)]
	pub type Lock<T: Config> = StorageMap<_, Twox64Concat, (CollectionId, NftId), bool, ValueQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_uniques::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type ProtocolOrigin: EnsureOrigin<Self::Origin>;
		type MaxRecursions: Get<u32>;

		#[pallet::constant]
		type ResourceSymbolLimit: Get<u32>;

		#[pallet::constant]
		type PartsLimit: Get<u32>;

		#[pallet::constant]
		type MaxPriorities: Get<u32>;
		type CollectionSymbolLimit: Get<u32>;
		type MaxResourcesOnMint: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CollectionCreated {
			issuer: T::AccountId,
			collection_id: CollectionId,
		},
		// NftMinted(T::AccountId, CollectionId, NftId),
		NftMinted {
			owner: T::AccountId,
			collection_id: CollectionId,
			nft_id: NftId,
		},
		NFTBurned {
			owner: T::AccountId,
			nft_id: NftId,
		},
		CollectionDestroyed {
			issuer: T::AccountId,
			collection_id: CollectionId,
		},
		NFTSent {
			sender: T::AccountId,
			recipient: AccountIdOrCollectionNftTuple<T::AccountId>,
			collection_id: CollectionId,
			nft_id: NftId,
			approval_required: bool,
		},
		NFTAccepted {
			sender: T::AccountId,
			recipient: AccountIdOrCollectionNftTuple<T::AccountId>,
			collection_id: CollectionId,
			nft_id: NftId,
		},
		NFTRejected {
			sender: T::AccountId,
			collection_id: CollectionId,
			nft_id: NftId,
		},
		IssuerChanged {
			old_issuer: T::AccountId,
			new_issuer: T::AccountId,
			collection_id: CollectionId,
		},
		PropertySet {
			collection_id: CollectionId,
			maybe_nft_id: Option<NftId>,
			key: KeyLimitOf<T>,
			value: ValueLimitOf<T>,
		},
		CollectionLocked {
			issuer: T::AccountId,
			collection_id: CollectionId,
		},
		ResourceAdded {
			nft_id: NftId,
			resource_id: ResourceId,
		},
		ResourceAccepted {
			nft_id: NftId,
			resource_id: ResourceId,
		},
		ResourceRemoval {
			nft_id: NftId,
			resource_id: ResourceId,
		},
		ResourceRemovalAccepted {
			nft_id: NftId,
			resource_id: ResourceId,
		},
		PrioritySet {
			collection_id: CollectionId,
			nft_id: NftId,
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
		where T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
	{
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn create_collection(
			origin: OriginFor<T>,
			metadata: BoundedVec<u8, T::StringLimit>,
			max: Option<u32>,
			symbol: BoundedCollectionSymbolOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let collection_id = Self::collection_create(sender.clone(), metadata, max, symbol)?;

			pallet_uniques::Pallet::<T>::do_create_collection(
				collection_id,
				sender.clone(),
				sender.clone(),
				T::CollectionDeposit::get(),
				false,
				pallet_uniques::Event::Created {
					collection: collection_id,
					creator: sender.clone(),
					owner: sender.clone(),
				},
			)?;

			Self::deposit_event(Event::CollectionCreated { issuer: sender, collection_id });
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn set_property(
			origin: OriginFor<T>,
			#[pallet::compact] collection_id: CollectionId,
			maybe_nft_id: Option<NftId>,
			key: KeyLimitOf<T>,
			value: ValueLimitOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			Self::property_set(sender, collection_id, maybe_nft_id, key.clone(), value.clone())?;

			Self::deposit_event(Event::PropertySet { collection_id, maybe_nft_id, key, value });
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn add_basic_resource(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
			resource: BasicResource<StringLimitOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let resource_id =
				Self::resource_add(sender, collection_id, nft_id, ResourceTypes::Basic(resource))?;

			Self::deposit_event(Event::ResourceAdded { nft_id, resource_id });
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn accept_resource(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
			resource_id: ResourceId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			ensure!(
				Resources::<T>::get((collection_id, nft_id, resource_id)).is_some(),
				Error::<T>::ResourceDoesntExist
			);

			let (owner, _) = Pallet::<T>::lookup_root_owner(collection_id, nft_id)?;
			ensure!(owner == sender, Error::<T>::NoPermission);

			Resources::<T>::try_mutate_exists(
				(collection_id, nft_id, resource_id),
				|resource| -> DispatchResult {
					if let Some(res) = resource.into_mut() {
						ensure!(res.pending, Error::<T>::ResourceNotPending);
						res.pending = false;
					}
					Ok(())
				},
			)?;

			Self::deposit_event(Event::ResourceAccepted { nft_id, resource_id });
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn remove_resource(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
			resource_id: ResourceId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			Self::resource_remove(sender, collection_id, nft_id, resource_id)?;

			Self::deposit_event(Event::ResourceRemoval { nft_id, resource_id });
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn accept_resource_removal(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
			resource_id: ResourceId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			Self::accept_removal(sender, collection_id, nft_id, resource_id)?;

			Self::deposit_event(Event::ResourceRemovalAccepted { nft_id, resource_id });
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn set_priority(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
			priorities: BoundedVec<ResourceId, T::MaxPriorities>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::priority_set(sender, collection_id, nft_id, priorities)?;
			Self::deposit_event(Event::PrioritySet { collection_id, nft_id });
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn destroy_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			Self::collection_burn(sender.clone(), collection_id)?;

			let witness = pallet_uniques::Pallet::<T>::get_destroy_witness(&collection_id)
				.ok_or(Error::<T>::NoWitness)?;
			ensure!(witness.items == 0u32, Error::<T>::CollectionNotEmpty);

			pallet_uniques::Pallet::<T>::do_destroy_collection(
				collection_id,
				witness,
				sender.clone().into(),
			)?;

			Self::deposit_event(Event::CollectionDestroyed { issuer: sender, collection_id });
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn lock_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let collection_id = Self::collection_lock(sender.clone(), collection_id)?;

			Self::deposit_event(Event::CollectionLocked { issuer: sender, collection_id });
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn change_collection_issuer(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			new_issuer: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			let collection =
				Self::collections(collection_id).ok_or(Error::<T>::CollectionUnknown)?;
			ensure!(collection.issuer == sender, Error::<T>::NoPermission);
			let new_owner = T::Lookup::lookup(new_issuer.clone())?;

			ensure!(
				Collections::<T>::contains_key(collection_id),
				Error::<T>::NoAvailableCollectionId
			);

			let (new_owner, collection_id) =
				Self::collection_change_issuer(collection_id, new_owner)?;

			pallet_uniques::Pallet::<T>::transfer_ownership(origin, collection_id, new_issuer)?;

			Self::deposit_event(Event::IssuerChanged {
				old_issuer: sender,
				new_issuer: new_owner,
				collection_id,
			});
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn mint_nft(
			origin: OriginFor<T>,
			owner: Option<T::AccountId>,
			collection_id: CollectionId,
			metadata: BoundedVec<u8, T::StringLimit>,
			// transferable: bool,
			// resources: Option<BoundedResourceTypeOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			if let Some(collection_issuer) =
			pallet_uniques::Pallet::<T>::collection_owner(collection_id)
			{
				ensure!(collection_issuer == sender, Error::<T>::NoPermission);
			} else {
				return Err(Error::<T>::CollectionUnknown.into())
			}

			let nft_owner = match owner {
				Some(owner) => owner,
				None => sender.clone(),
			};

			let (collection_id, nft_id) = Self::nft_mint(
				sender,
				nft_owner.clone(),
				collection_id,
				metadata,
				true,
			)?;

			pallet_uniques::Pallet::<T>::do_mint(
				collection_id,
				nft_id,
				nft_owner.clone(),
				|_details| Ok(()),
			)?;

			// if let Some(resources) = resources {
			// 	for res in resources {
			// 		Self::resource_add(nft_owner.clone(), collection_id, nft_id, res)?;
			// 	}
			// }

			Self::deposit_event(Event::NftMinted { owner: nft_owner, collection_id, nft_id });

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn burn_nft(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			let (root_owner, _) = Pallet::<T>::lookup_root_owner(collection_id, nft_id)?;
			// Check ownership
			ensure!(sender == root_owner, Error::<T>::NoPermission);
			let max_recursions = T::MaxRecursions::get();
			let (_collection_id, nft_id) = Self::nft_burn(collection_id, nft_id, max_recursions)?;

			pallet_uniques::Pallet::<T>::do_burn(collection_id, nft_id, |_, _| Ok(()))?;

			Self::deposit_event(Event::NFTBurned { owner: sender, nft_id });
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn send(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
			new_owner: AccountIdOrCollectionNftTuple<T::AccountId>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let (new_owner_account, approval_required) =
				Self::nft_send(sender.clone(), collection_id, nft_id, new_owner.clone())?;

			pallet_uniques::Pallet::<T>::do_transfer(
				collection_id,
				nft_id,
				new_owner_account,
				|_class_details, _details| Ok(()),
			)?;

			Self::deposit_event(Event::NFTSent {
				sender,
				recipient: new_owner.clone(),
				collection_id,
				nft_id,
				approval_required,
			});

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn accept_nft(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
			new_owner: AccountIdOrCollectionNftTuple<T::AccountId>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let (new_owner_account, collection_id, nft_id) =
				Self::nft_accept(sender.clone(), collection_id, nft_id, new_owner.clone())?;

			pallet_uniques::Pallet::<T>::do_transfer(
				collection_id,
				nft_id,
				new_owner_account,
				|_class_details, _details| Ok(()),
			)?;

			Self::deposit_event(Event::NFTAccepted {
				sender,
				recipient: new_owner.clone(),
				collection_id,
				nft_id,
			});
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn reject_nft(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let max_recursions = T::MaxRecursions::get();
			let (sender, collection_id, nft_id) =
				Self::nft_reject(sender, collection_id, nft_id, max_recursions)?;

			Self::deposit_event(Event::NFTRejected { sender, collection_id, nft_id });
			Ok(())
		}

	}
}
