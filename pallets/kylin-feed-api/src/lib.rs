#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::too_many_arguments)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use frame_support::{
	dispatch::{GetDispatchInfo, DispatchResult},
	ensure, traits::tokens::nonfungibles::*, BoundedVec,
	traits::UnixTime,
	pallet_prelude::*,
	log,
};
use frame_system::{
	ensure_signed,
    Config as SystemConfig,
};

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
use sp_std::{prelude::*, str, vec::Vec};
use codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use cumulus_pallet_xcm::{ensure_sibling_para, Origin as CumulusOrigin};
use cumulus_primitives_core::ParaId;
use xcm::latest::{prelude::*, Junction, OriginKind, SendXcm, Xcm};

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

#[derive(Encode, Decode)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, TypeInfo,)]
pub struct MetaData {
    key: Vec<u8>,
    url: Vec<u8>,
	vpath: Vec<u8>,
}

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone, Copy, TypeInfo, MaxEncodedLen)]
pub struct TimestampedValue {
    pub value: i64,
    pub timestamp: u128,
}

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

	/// NFT ID tracker, increased after new NFT minted
	#[pallet::storage]
	#[pallet::getter(fn next_nft_id)]
	pub type NextNftId<T: Config> = StorageMap<_,
		Twox64Concat, CollectionId,
		NftId, ValueQuery>;

	/// Collection ID tracker, increased after new collection created
	#[pallet::storage]
	#[pallet::getter(fn collection_index)]
	pub type CollectionIndex<T: Config> = StorageValue<_, CollectionId, ValueQuery>;

	/// Resource ID tracker, increased after new resource created
	#[pallet::storage]
	#[pallet::getter(fn next_resource_id)]
	pub type NextResourceId<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat, CollectionId,
		Twox64Concat, NftId,
		ResourceId, ValueQuery>;

	/// Storage map for collection infomation 
	#[pallet::storage]
	#[pallet::getter(fn collections)]
	pub type Collections<T: Config> = StorageMap<
		_,
		Twox64Concat, CollectionId,
		CollectionInfo<StringLimitOf<T>, BoundedCollectionSymbolOf<T>, T::AccountId>,
	>;

	/// Storage map for NFT infomation 
	#[pallet::storage]
	#[pallet::getter(fn nfts)]
	pub type Nfts<T: Config> =
	StorageDoubleMap<_,
		Twox64Concat, CollectionId,
		Twox64Concat, NftId,
		InstanceInfoOf<T>>;

	/// Storage map for Priorities infomation 
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

	/// Storage tree for NFTs
	#[pallet::storage]
	#[pallet::getter(fn children)]
	pub type Children<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat, (CollectionId, NftId),
		Twox64Concat, (CollectionId, NftId),
		(),
	>;

	/// Storage map for resource 
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

	/// Storage map for Properties 
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

	/// Collection operation lock
	#[pallet::storage]
	#[pallet::getter(fn lock)]
	pub type Lock<T: Config> = StorageMap<_, Twox64Concat, (CollectionId, NftId), bool, ValueQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_uniques::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		//type ProtocolOrigin: EnsureOrigin<Self::Origin>;
		type RuntimeOrigin: From<<Self as SystemConfig>::RuntimeOrigin>
			+ Into<Result<CumulusOrigin, <Self as Config>::RuntimeOrigin>>;
		type MaxRecursions: Get<u32>;
		type UnixTime: UnixTime;

		#[pallet::constant]
		type ResourceSymbolLimit: Get<u32>;

		#[pallet::constant]
		type PartsLimit: Get<u32>;

		#[pallet::constant]
		type MaxPriorities: Get<u32>;
		type CollectionSymbolLimit: Get<u32>;
		type MaxResourcesOnMint: Get<u32>;
		type XcmSender: SendXcm;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CollectionCreated {
			issuer: T::AccountId,
			collection_id: CollectionId,
		},
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
		FeedCreated {
			owner: T::AccountId,
			collection_id: CollectionId,
			nft_id: NftId,
			metadata: MetaData,
		},
		FeedRemoved {
			owner: T::AccountId,
			nft_id: NftId,
		},
		QueryFeedBack {
			key: Vec<u8>,
			value: TimestampedValue,
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
		JsonError,
		XcmSendError,
	}


	#[pallet::call]
	impl<T: Config> Pallet<T>
		where T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId> + 
		kylin_oracle::Config,
		<T as frame_system::Config>::AccountId: AsRef<[u8]>,
	{
		/// Create a new collection, a container for user NFTs.
		///
		/// Can be called by any signed origin.
		///
		/// # Parameter:
		/// * `metadata` - metadata for the collection
		/// * `max` - max count for NFT
		/// * `symbol` - symbol for the collection
		/// 
		/// # Emits
		/// * `CollectionCreated`
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
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

		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn xcm_create_collection(
			origin: OriginFor<T>,
			metadata: BoundedVec<u8, T::StringLimit>,
			max: Option<u32>,
			symbol: BoundedCollectionSymbolOf<T>,
		) -> DispatchResult {
			let para_id =
                ensure_sibling_para(<T as Config>::RuntimeOrigin::from(origin.clone()))?;
			let sender = Self::paraid_to_account_id::<T::AccountId>(para_id);

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

			Self::sendback_collectionid(para_id, collection_id)?;
			Self::deposit_event(Event::CollectionCreated { issuer: sender, collection_id });
			Ok(())
		}

		/// Set the property for the collection, a container for user NFTs.
		///
		/// Can be called only by the collection issuer.
		///
		/// # Parameter:
		/// * `collection_id` - collection ID
		/// * `maybe_nft_id` - NFT ID, optional
		/// * `key` - key
		/// * `value` - property value
		/// 
		/// # Emits
		/// * `PropertySet`
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
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

		/// Destroy the collection.
		///
		/// Can be called only by the collection issuer.
		///
		/// # Parameter:
		/// * `collection_id` - collection ID
		/// 
		/// # Emits
		/// * `CollectionDestroyed`
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn destroy_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			Self::core_destroy_collection(sender, collection_id)?;
			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn xcm_destroy_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResult {
			let para_id =
                ensure_sibling_para(<T as Config>::RuntimeOrigin::from(origin.clone()))?;
			let sender = Self::paraid_to_account_id::<T::AccountId>(para_id);

			Self::core_destroy_collection(sender, collection_id)?;
			Ok(())
		}
		/// Lock the collection, so as to suspend operations.
		///
		/// Can be called only by the collection issuer.
		///
		/// # Parameter:
		/// * `collection_id` - collection ID
		/// 
		/// # Emits
		/// * `CollectionLocked`
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn lock_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let collection_id = Self::collection_lock(sender.clone(), collection_id)?;

			Self::deposit_event(Event::CollectionLocked { issuer: sender, collection_id });
			Ok(())
		}

		/// Change the collection issuer.
		///
		/// Can be called only by the collection issuer.
		///
		/// # Parameter:
		/// * `collection_id` - collection ID
		/// * `new_issuer` - new issuer
		/// 
		/// # Emits
		/// * `IssuerChanged`
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
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

		/// Create new feed with a NFT attached in the specific collection
		///
		/// Can be called only by the collection issuer.
		///
		/// # Parameter:
		/// * `collection_id` - collection ID
		/// * `oracle_paraid` - parachain ID of the Oracle chain
		/// * `key` - key for the feed
		/// * `url` - url for the feed
		/// * `vpath` - value path of the URL result
		///     example: json = {"x":{"y": ["z", "zz"]}}
        ///     path: "/x/y/1" = "zz" 
		/// 
		/// # Emits
		/// * `FeedCreated`
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn create_feed(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			oracle_paraid: u32,
			key: Vec<u8>,
            url: Vec<u8>,
			vpath: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			
			if let Some(collection_issuer) =
			pallet_uniques::Pallet::<T>::collection_owner(collection_id)
			{
				ensure!(collection_issuer == sender, Error::<T>::NoPermission);
			} else {
				return Err(Error::<T>::CollectionUnknown.into())
			}

			let key_limit: kylin_oracle::OracleKeyOf<T> = key.clone().try_into().map_err(
				|_| Error::<T>::StorageOverflow
			)?;
			kylin_oracle::Pallet::<T>::submit_api(origin, key_limit, url.clone(), vpath.clone())?;

			let mdata = MetaData { key, url, vpath };
			let meta_str = serde_json::to_string(&mdata).map_err(|_| Error::<T>::JsonError)?;
			let metadata: BoundedVec<u8, T::StringLimit> = meta_str.as_bytes().to_vec()
				.try_into().map_err(
				|_| Error::<T>::StorageOverflow
			)?;

			let nft_owner = sender.clone();
			let (collection_id, nft_id) = Self::nft_mint(
				sender,
				nft_owner.clone(),
				collection_id,
				metadata.clone(),
				true,
			)?;

			pallet_uniques::Pallet::<T>::do_mint(
				collection_id,
				nft_id,
				nft_owner.clone(),
				|_details| Ok(()),
			)?;

			Self::deposit_event(Event::FeedCreated { owner: nft_owner, collection_id, nft_id, metadata:mdata });
			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn xcm_create_feed(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			key: Vec<u8>,
            url: Vec<u8>,
			vpath: Vec<u8>,
		) -> DispatchResult {
			let para_id =
                ensure_sibling_para(<T as Config>::RuntimeOrigin::from(origin.clone()))?;
			let sender = Self::paraid_to_account_id::<T::AccountId>(para_id);
			
			if let Some(collection_issuer) =
			pallet_uniques::Pallet::<T>::collection_owner(collection_id)
			{
				ensure!(collection_issuer == sender, Error::<T>::NoPermission);
			} else {
				return Err(Error::<T>::CollectionUnknown.into())
			}

			let key_limit: kylin_oracle::OracleKeyOf<T> = key.clone().try_into().map_err(
				|_| Error::<T>::StorageOverflow
			)?;
			kylin_oracle::Pallet::<T>::xcm_submit_api(origin, key_limit, url.clone(), vpath.clone())?;

			let mdata = MetaData { key, url, vpath };
			let meta_str = serde_json::to_string(&mdata).map_err(|_| Error::<T>::JsonError)?;
			let metadata: BoundedVec<u8, T::StringLimit> = meta_str.as_bytes().to_vec()
				.try_into().map_err(
				|_| Error::<T>::StorageOverflow
			)?;

			let nft_owner = sender.clone();
			let (collection_id, nft_id) = Self::nft_mint(
				sender,
				nft_owner.clone(),
				collection_id,
				metadata.clone(),
				true,
			)?;

			pallet_uniques::Pallet::<T>::do_mint(
				collection_id,
				nft_id,
				nft_owner.clone(),
				|_details| Ok(()),
			)?;

			Self::sendback_nftid(para_id, collection_id, nft_id)?;
			Self::deposit_event(Event::FeedCreated { owner: nft_owner, collection_id, nft_id, metadata:mdata });
			Ok(())
		}

		/// Remove a feed
		///
		/// Can be called only by the feed owner.
		///
		/// # Parameter:
		/// * `collection_id` - collection ID
		/// * `nft_id` - nft ID
		/// 
		/// # Emits
		/// * `FeedRemoved`
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn remove_feed(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			
			let (root_owner, _) = Pallet::<T>::lookup_root_owner(collection_id, nft_id)?;
			// Check ownership
			ensure!(sender == root_owner, Error::<T>::NoPermission);

			let nft = Nfts::<T>::get(collection_id, nft_id).ok_or(Error::<T>::NoAvailableNftId)?;
			let mdata: MetaData = serde_json::from_slice(&nft.metadata).map_err(|_| Error::<T>::JsonError)?;

			let key_limit: kylin_oracle::OracleKeyOf<T> = mdata.key.clone().try_into().map_err(
					|_| Error::<T>::StorageOverflow
				)?;
			kylin_oracle::Pallet::<T>::remove_api(origin, key_limit)?;

			let max_recursions = T::MaxRecursions::get();
			let (_collection_id, nft_id) = Self::nft_burn(collection_id, nft_id, max_recursions)?;

			pallet_uniques::Pallet::<T>::do_burn(collection_id, nft_id, |_, _| Ok(()))?;

			Self::deposit_event(Event::FeedRemoved { owner: sender, nft_id });
			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn xcm_remove_feed(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
		) -> DispatchResult {
			let para_id =
                ensure_sibling_para(<T as Config>::RuntimeOrigin::from(origin.clone()))?;
			let sender = Self::paraid_to_account_id::<T::AccountId>(para_id);

			let (root_owner, _) = Pallet::<T>::lookup_root_owner(collection_id, nft_id)?;
			// Check ownership
			ensure!(sender == root_owner, Error::<T>::NoPermission);

			let nft = Nfts::<T>::get(collection_id, nft_id).ok_or(Error::<T>::NoAvailableNftId)?;
			let mdata: MetaData = serde_json::from_slice(&nft.metadata).map_err(|_| Error::<T>::JsonError)?;

			let key_limit: kylin_oracle::OracleKeyOf<T> = mdata.key.clone().try_into().map_err(
					|_| Error::<T>::StorageOverflow
				)?;
			kylin_oracle::Pallet::<T>::xcm_remove_api(origin, key_limit)?;

			let max_recursions = T::MaxRecursions::get();
			let (_collection_id, nft_id) = Self::nft_burn(collection_id, nft_id, max_recursions)?;

			pallet_uniques::Pallet::<T>::do_burn(collection_id, nft_id, |_, _| Ok(()))?;

			Self::deposit_event(Event::FeedRemoved { owner: sender, nft_id });
			Ok(())
		}
		
		/// Query the feed data
		///
		/// Can be called only by the feed owner.
		///
		/// # Parameter:
		/// * `collection_id` - collection ID
		/// * `nft_id` - NFT ID
		/// 
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn xcm_query_feed(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
		) -> DispatchResult {
			let para_id =
                ensure_sibling_para(<T as Config>::RuntimeOrigin::from(origin.clone()))?;
			let sender = Self::paraid_to_account_id::<T::AccountId>(para_id);
			let (root_owner, _) = Pallet::<T>::lookup_root_owner(collection_id, nft_id)?;
			// Check ownership
			ensure!(sender == root_owner, Error::<T>::NoPermission);

			let nft = Nfts::<T>::get(collection_id, nft_id).ok_or(Error::<T>::NoAvailableNftId)?;
			let mdata: MetaData = serde_json::from_slice(&nft.metadata).map_err(|_| Error::<T>::JsonError)?;
			
			let key: kylin_oracle::OracleKeyOf<T> = mdata.key.clone().try_into().map_err(
				|_| Error::<T>::StorageOverflow
			)?;
			if let Some(val) = kylin_oracle::Pallet::<T>::get(&key) {
                Self::sendback_query_res(para_id, mdata.key, val.value)
            } else {
                Err(DispatchError::CannotLookup)
            }
		}
		
		/// Transfer the ownership of the NFT
		///
		/// Can be called only by the NFT owner.
		///
		/// # Parameter:
		/// * `collection_id` - collection ID
		/// * `nft_id` - nft ID
		/// * `new_owner` - new owner
		/// 
		/// # Emits
		/// * `NFTSent`
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
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

		/// Accept the ownership of the transfered NFT
		///
		/// Can be called only by the NFT root owner.
		///
		/// # Parameter:
		/// * `collection_id` - collection ID
		/// * `nft_id` - nft ID
		/// * `new_owner` - new owner
		/// 
		/// # Emits
		/// * `NFTAccepted`
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
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

		/// Reject the ownership of the transfered NFT
		///
		/// Can be called only by the NFT root owner.
		///
		/// # Parameter:
		/// * `collection_id` - collection ID
		/// * `nft_id` - nft ID
		/// 
		/// # Emits
		/// * `NFTRejected`
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
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

impl<T: Config> Pallet<T>
    where T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
{
	pub fn core_destroy_collection(
		sender: T::AccountId,
		collection_id: CollectionId,
	) -> DispatchResult {

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
	
}