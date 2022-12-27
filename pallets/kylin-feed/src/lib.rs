#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::too_many_arguments)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use codec::{Decode, Encode};
use frame_support::{
	dispatch::DispatchResult, ensure, traits::tokens::nonfungibles::*, BoundedVec,
	traits::UnixTime,
	log,
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
use cumulus_primitives_core::ParaId;
use xcm::latest::{prelude::*, Junction, OriginKind, SendXcm, Xcm};

use kylin_primitives::types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use pallet::*;

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone, Copy, TypeInfo, MaxEncodedLen)]
pub struct TimestampedValue {
    pub value: i64,
    pub timestamp: u128,
}

/// Mock structure for XCM Call message encoding
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[allow(non_camel_case_types)]
enum KylinFeedApiFunc {
    #[codec(index = 1u8)]
    xcm_create_collection { 
		metadata: Vec<u8>,
		max: Option<u32>,
		symbol: Vec<u8>,
    },
	#[codec(index = 1u8)]
	xcm_destroy_collection { 
		collection_id: u32,
    },
	#[codec(index = 1u8)]
	xcm_create_feed { 
		collection_id: u32,
		key: Vec<u8>,
		url: Vec<u8>,
		vpath: Vec<u8>,
    },
	#[codec(index = 1u8)]
	xcm_remove_feed { 
		collection_id: u32,
		nft_id: u32,
    },
    #[codec(index = 2u8)]
    xcm_query_feed { 
        collection_id: u32,
		nft_id: u32,
    },
}

/// Mock structure for XCM Call message encoding
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[allow(non_camel_case_types)]
enum KylinXcmCall {
    #[codec(index = 167u8)]
    KylinFeedApi(KylinFeedApiFunc),
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub(crate) type KeyLimitOf<T> = BoundedVec<u8, <T as Config>::StringLimit>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
    #[pallet::getter(fn values)]
    pub type Values<T: Config> = StorageMap<_, Twox64Concat, KeyLimitOf<T>, TimestampedValue>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeOrigin: From<<Self as SystemConfig>::RuntimeOrigin>
            + Into<Result<CumulusOrigin, <Self as Config>::RuntimeOrigin>>;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type UnixTime: UnixTime;

		#[pallet::constant]
		type StringLimit: Get<u32>;

		type XcmSender: SendXcm;
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
		XcmSendError,
	}


	#[pallet::call]
	impl<T: Config> Pallet<T>
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
			oracle_paraid: ParaId,
			metadata: BoundedVec<u8, T::StringLimit>,
			max: Option<u32>,
			symbol: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			
			Self::do_create_collection(oracle_paraid, metadata, max, symbol)?;
			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn xcm_return_collectionid(
			origin: OriginFor<T>, 
			collection_id: CollectionId
		) -> DispatchResult {
            let para_id = ensure_sibling_para(<T as Config>::RuntimeOrigin::from(origin.clone()))?;

            Self::deposit_event(Event::CollectionCreated { issuer: sender, collection_id });
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
			oracle_paraid: ParaId,
			collection_id: CollectionId,
			key: Vec<u8>,
            url: Vec<u8>,
			vpath: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			
			Self::do_create_feed(oracle_paraid, collection_id, &key, &url, &vpath)?;
			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn xcm_return_nftid(
			origin: OriginFor<T>, 
			collection_id: CollectionId,
			nft_id: NftId,
		) -> DispatchResult {
            let para_id = ensure_sibling_para(<T as Config>::RuntimeOrigin::from(origin.clone()))?;

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
			oracle_paraid: ParaId,
			collection_id: CollectionId,
			nft_id: NftId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			
			
			Self::do_remove_feed(oracle_paraid, collection_id， nft_id)?;
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
		pub fn query_feed(
			origin: OriginFor<T>,
			oracle_paraid: ParaId,
			collection_id: CollectionId,
			nft_id: NftId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			
			Self::do_query_feed(oracle_paraid, collection_id, nft_id)?;
			Ok(())
		}

		/// Feed data query feed back from Oracle parachain
		///
		/// Can be only XCM call from parachain.
		///
		/// # Parameter:
		/// * `key` - key for the feed
		/// * `value` - value for the feed
		/// 
		/// # Emits
		/// * `QueryFeedBack`
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn xcm_feed_back(origin: OriginFor<T>, key: Vec<u8>, value: i64) -> DispatchResult {
            let para_id = ensure_sibling_para(<T as Config>::RuntimeOrigin::from(origin.clone()))?;

            let now = T::UnixTime::now().as_millis();
            let tval = TimestampedValue {
                value: value.clone(),
                timestamp: now,
            };

            let keylimit: KeyLimitOf<T> = key.clone().try_into().map_err(|_| Error::<T>::StorageOverflow)?;
            <Values<T>>::insert(keylimit, tval);
            Self::deposit_event(Event::QueryFeedBack { key, value: tval });
            Ok(())
        }

	}
}

impl<T: Config> Pallet<T>
{
	pub fn do_create_collection(
		para_id: u32,
		metadata: BoundedVec<u8, T::StringLimit>,
		max: Option<u32>,
		symbol: BoundedVec<u8, T::StringLimit>,
	) -> DispatchResult {
        let remark = KylinXcmCall::KylinFeedApi(KylinFeedApiFunc::xcm_create_collection{
            metadata.to_vec(), max, symbol.to_vec(),
        });
        T::XcmSender::send_xcm(
            (
                1,
                Junction::Parachain(para_id.into()),
            ),
            Xcm(vec![Transact {
                origin_type: OriginKind::Native,
                require_weight_at_most: 1_000_000_000,
                call: remark.encode().into(),
            }]),
        ).map_err(
            |e| {
                log::error!("Error: XcmSendError {:?}, {:?}", para_id, e);
                Error::<T>::XcmSendError
            }
        )?;

        Ok(())
    }

	pub fn do_destroy_collection(
		para_id: u32,
		collection_id: u32,
	) -> DispatchResult {
        let remark = KylinXcmCall::KylinFeedApi(KylinFeedApiFunc::xcm_destroy_collection{
            collection_id,
        });
        T::XcmSender::send_xcm(
            (
                1,
                Junction::Parachain(para_id.into()),
            ),
            Xcm(vec![Transact {
                origin_type: OriginKind::Native,
                require_weight_at_most: 1_000_000_000,
                call: remark.encode().into(),
            }]),
        ).map_err(
            |e| {
                log::error!("Error: XcmSendError {:?}, {:?}", para_id, e);
                Error::<T>::XcmSendError
            }
        )?;

        Ok(())
    }

	pub fn do_create_feed(
		para_id: u32,
		collection_id: CollectionId,
		key: Vec<u8>,
		url: Vec<u8>,
		vpath: Vec<u8>,
	) -> DispatchResult {
        let remark = KylinXcmCall::KylinFeedApi(KylinFeedApiFunc::xcm_create_feed{
            collection_id, key, url, vpath
        });
        T::XcmSender::send_xcm(
            (
                1,
                Junction::Parachain(para_id.into()),
            ),
            Xcm(vec![Transact {
                origin_type: OriginKind::Native,
                require_weight_at_most: 1_000_000_000,
                call: remark.encode().into(),
            }]),
        ).map_err(
            |e| {
                log::error!("Error: XcmSendError {:?}, {:?}", para_id, e);
                Error::<T>::XcmSendError
            }
        )?;

        Ok(())
    }

	pub fn do_remove_feed(
		para_id: u32,
		collection_id: u32,
		nft_id: u32,
	) -> DispatchResult {
        let remark = KylinXcmCall::KylinFeedApi(KylinFeedApiFunc::xcm_remove_feed{
            collection_id, nft_id
        });
        T::XcmSender::send_xcm(
            (
                1,
                Junction::Parachain(para_id.into()),
            ),
            Xcm(vec![Transact {
                origin_type: OriginKind::Native,
                require_weight_at_most: 1_000_000_000,
                call: remark.encode().into(),
            }]),
        ).map_err(
            |e| {
                log::error!("Error: XcmSendError {:?}, {:?}", para_id, e);
                Error::<T>::XcmSendError
            }
        )?;

        Ok(())
    }

	pub fn do_query_feed(para_id: u32, collection_id: CollectionId, nft_id: NftId) -> DispatchResult {
        let remark = KylinXcmCall::KylinFeedApi(KylinFeedApiFunc::xcm_query_feed{
            collection_id, nft_id
        });
        T::XcmSender::send_xcm(
            (
                1,
                Junction::Parachain(para_id.into()),
            ),
            Xcm(vec![Transact {
                origin_type: OriginKind::Native,
                require_weight_at_most: 1_000_000_000,
                call: remark.encode().into(),
            }]),
        ).map_err(
            |e| {
                log::error!("Error: XcmSendError {:?}, {:?}", para_id, e);
                Error::<T>::XcmSendError
            }
        )?;

        Ok(())
    }

	
}