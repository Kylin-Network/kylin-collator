#![cfg_attr(not(feature = "std"), no_std)]
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode};
use cumulus_pallet_xcm::{ensure_sibling_para, Origin as CumulusOrigin};
use cumulus_primitives_core::ParaId;
use frame_support::{
    dispatch::DispatchResultWithPostInfo,
    log,
    pallet_prelude::*,
    traits::{Currency, EstimateCallFee, UnixTime, ChangeMembers, Get, SortedMembers, Time},
    IterableStorageMap,
};
use frame_system::{
    self as system,
    offchain::{
        AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer, SubmitTransaction,
    },
    pallet_prelude::*,
    Config as SystemConfig,
};
use hex::ToHex;
use lite_json::{
    json::{JsonValue, NumberValue},
    Serialize as JsonSerialize,
};
use scale_info::TypeInfo;
use sp_std::{borrow::ToOwned, convert::TryFrom, convert::TryInto, prelude::*, str, vec, vec::Vec};

use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    offchain::{
        http,
        storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
        Duration,
    },
    traits::{Hash, UniqueSaturatedInto, Zero},
};
use xcm::latest::{prelude::*, Junction, OriginKind, SendXcm, Xcm};
use orml_traits::{CombineData, DataFeeder, DataProvider, DataProviderExtended, OnNewData};
use orml_utilities::OrderedSet;

pub use pallet::*;
#[cfg(test)]
mod tests;

mod ringbuffer;
use ringbuffer::{RingBufferTrait, RingBufferTransient};

// Runtime benchmarking features
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
/// ocpf mean off-chain worker price fetch
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocpf");
/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
pub mod crypto {
    use super::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
    };
    use sp_std::{convert::TryFrom};
    use sp_runtime::{MultiSignature, MultiSigner};
    app_crypto!(sr25519, KEY_TYPE);
    pub struct TestAuthId;
    // implemented for ocw-runtime
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for TestAuthId
    {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

/// An index to a block.
///

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    pub(crate) type MomentOf<T> = <<T as Config>::Time as Time>::Moment;
	pub(crate) type TimestampedValueOf<T> = TimestampedValue<<T as Config>::OracleValue, MomentOf<T>>;

	#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone, Copy, Ord, PartialOrd, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct TimestampedValue<Value, Moment> {
		pub value: Value,
		pub timestamp: Moment,
	}

    #[pallet::config]
    pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config + pallet_balances::Config
    where <Self as frame_system::Config>::AccountId: AsRef<[u8]> + ToHex
    {
        /// The identifier type for an offchain worker.
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

        type Origin: From<<Self as SystemConfig>::Origin>
            + Into<Result<CumulusOrigin, <Self as Config>::Origin>>;

        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The overarching dispatch call type.
        type Call: From<Call<Self>> + Encode;

        type XcmSender: SendXcm;

        type UnixTime: UnixTime;

        /// A configuration for base priority of unsigned transactions.
        ///
        /// This is exposed so that it can be tuned for particular runtime, when
        /// multiple pallets send unsigned transactions.
        #[pallet::constant]
        type UnsignedPriority: Get<TransactionPriority>;

        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;

        type EstimateCallFee: EstimateCallFee<Call<Self>, BalanceOf<Self>>;

        type Currency: frame_support::traits::Currency<Self::AccountId>;

        /// Provide the implementation to combine raw values to produce
		/// aggregated value
		type CombineData: CombineData<Self::OracleKey, TimestampedValueOf<Self>>;

		/// Time provider
		type Time: Time;

        /// The data key type
		type OracleKey: Parameter + Member + MaxEncodedLen;

		/// The data value type
		type OracleValue: Parameter + Member + Ord + MaxEncodedLen;

        /// Oracle operators.
		type Members: SortedMembers<Self::AccountId>;

		/// Maximum size of HasDispatched
		#[pallet::constant]
		type MaxHasDispatchedSize: Get<u32>;

    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

	#[pallet::error]
    pub enum Error<T> {
        /// DataRequest Fields is too large to store on-chain.
        TooLarge,
        /// Sender does not have permission
		NoPermission,
		/// Feeder has already feeded at this block
		AlreadyFeeded,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        T::AccountId: AsRef<[u8]> + ToHex + Decode
    {
        /// `on_initialize` to return the weight used in `on_finalize`.
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			<T as Config>::WeightInfo::on_finalize()
		}

		fn on_finalize(_n: T::BlockNumber) {
			// cleanup for next block
			<HasDispatched<T>>::kill();
		}

        fn offchain_worker(block_number: T::BlockNumber) {
            // Note that having logs compiled to WASM may cause the size of the blob to increase
            // significantly. You can use `RuntimeDebug` custom derive to hide details of the types
            // in WASM. The `sp-api` crate also provides a feature `disable-logging` to disable
            // all logging and thus, remove any logging from the WASM.

            let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
            log::debug!(
                "Current block: {:?} (parent hash: {:?})",
                block_number,
                parent_hash
            );

            // It's a good practice to keep `fn offchain_worker()` function minimal, and move most
            // of the code to separate `impl` block.
            // Here we call a helper function to calculate current average price.
            // This function reads storage entries of the current state.

            let should_send = Self::choose_transaction_type(block_number);
            let res = match should_send {
                TransactionType::Signed => Self::fetch_data_and_send_signed(block_number),
                TransactionType::Raw
                | TransactionType::UnsignedForAll
                | TransactionType::UnsignedForAny => {
                    Self::fetch_data_and_send_raw_unsigned(block_number)
                }
                _ => Ok(()),
            };
            if let Err(e) = res {
                log::error!("Error: {}", e);
            }
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        T::AccountId: AsRef<[u8]> + ToHex + Decode
    {
        #[pallet::weight(<T as Config>::WeightInfo::clear_api_queue_unsigned())]
        pub fn clear_api_queue_unsigned(
            origin: OriginFor<T>,
            _block_number: T::BlockNumber,
            processed_requests: Vec<u64>,
        ) -> DispatchResultWithPostInfo {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;
            for key in processed_requests.iter() {
                <ApiQueue<T>>::remove(&key);
            }
            Ok(().into())
        }
        
        #[pallet::weight(0 + T::DbWeight::get().writes(1))]
        pub fn clear_processed_requests_unsigned(
            origin: OriginFor<T>,
            _block_number: T::BlockNumber,
            processed_requests: Vec<u64>,
        ) -> DispatchResultWithPostInfo {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;
            let block_number = <system::Pallet<T>>::block_number();
            let current_timestamp = T::UnixTime::now().as_millis();
            for key in processed_requests.iter() {
                if SavedRequests::<T>::contains_key(key.clone()) {
                    let saved_request = Self::saved_data_requests(key).unwrap();
                    let processed_request = DataRequest {
                        para_id: saved_request.para_id,
                        account_id: saved_request.account_id,
                        feed_name: saved_request.feed_name.clone(),
                        requested_block_number: saved_request.requested_block_number,
                        processed_block_number: Some(block_number),
                        requested_timestamp: saved_request.requested_timestamp,
                        processed_timestamp: Some(current_timestamp),
                        payload: saved_request.payload.clone(),
                        is_query: saved_request.is_query,
                        url: saved_request.url.clone(),
                    };

                    <SavedRequests<T>>::insert(key, processed_request.clone());

                    let encoded_hash = hex::encode(
                        sp_runtime::traits::BlakeTwo256::hash(
                            processed_request.clone().encode().as_slice(),
                        )
                        .as_bytes(),
                    )
                    .as_bytes()
                    .to_vec();
                    if processed_request.is_query {
                        Self::deposit_event(Event::ReadFromDWH(
                            processed_request.para_id,
                            encoded_hash.clone(),
                            processed_request.feed_name.clone(),
                            processed_request.clone(),
                            processed_request.processed_block_number.clone().unwrap(),
                        ));
                    } else {
                        //insert to api
                        <ApiQueue<T>>::insert(key, processed_request.clone());
                        let feed_owner =
                            Self::feed_account_lookup(processed_request.feed_name.clone()).unwrap().0;
                        <FeedAccountLookup<T>>::insert(
                            processed_request.feed_name.clone(),
                            (&feed_owner, encoded_hash.clone()),
                        );
                        Self::deposit_event(Event::SavedToDWH(
                            processed_request.para_id,
                            encoded_hash.clone(),
                            processed_request.feed_name.clone(),
                            processed_request.clone(),
                            processed_request.processed_block_number.clone().unwrap(),
                        ));
                    }

                    <DataRequests<T>>::remove(&key);
                    <NextUnsignedAt<T>>::put(block_number);
                }
            }
            Ok(().into())
        }

        #[pallet::weight(<T as Config>::WeightInfo::query_data())]
        pub fn query_data(
            origin: OriginFor<T>,
            para_id: Option<ParaId>,
            feed_name: Vec<u8>,
        ) -> DispatchResult {
            ensure_signed(origin.clone())?;
            let submitter_account_id = ensure_signed(origin.clone())?;
            Self::query_feed(submitter_account_id, para_id, feed_name)
        }

        #[pallet::weight(0)]
        pub fn receive_response_from_parachain(
            origin: OriginFor<T>,
            feed_name: Vec<u8>,
            response: Vec<u8>,
        ) -> DispatchResult {
            let para_id = ensure_sibling_para(<T as Config>::Origin::from(origin))?;
            let block_number = <system::Pallet<T>>::block_number();
            Self::deposit_event(Event::ResponseReceived(
                para_id,
                feed_name.clone(),
                response.clone(),
                block_number,
            ));
            Ok(())
        }
        
        #[pallet::weight(<T as Config>::WeightInfo::submit_data_signed())]
        pub fn submit_data_signed(
            origin: OriginFor<T>,
            block_number: T::BlockNumber,
            key: u64,
            data: Vec<u8>,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            ensure_signed(origin.clone())?;
            let submitter_account_id = ensure_signed(origin.clone())?;

            let data_request = Self::data_requests(key).unwrap();
            let saved_request = DataRequest {
                para_id: data_request.para_id,
                account_id: Some(submitter_account_id),
                feed_name: data_request.feed_name.clone(),
                requested_block_number: data_request.requested_block_number,
                processed_block_number: Some(block_number),
                requested_timestamp: data_request.requested_timestamp,
                processed_timestamp: None,
                payload: data,
                is_query: data_request.is_query,
                url: data_request.url.clone(),
            };

            Self::save_data_response_onchain(block_number, key, saved_request);
            Ok(())
        }

        #[pallet::weight(<T as Config>::WeightInfo::submit_data_unsigned())]
        pub fn submit_data_unsigned(
            origin: OriginFor<T>,
            block_number: T::BlockNumber,
            key: u64,
            data: Vec<u8>,
        ) -> DispatchResult {
            ensure_none(origin.clone())?;
            let data_request = Self::data_requests(key).unwrap();
            let saved_request = DataRequest {
                para_id: data_request.para_id,
                account_id: data_request.account_id,
                feed_name: data_request.feed_name.clone(),
                requested_block_number: data_request.requested_block_number,
                processed_block_number: Some(block_number),
                requested_timestamp: data_request.requested_timestamp,
                processed_timestamp: None,
                payload: data,
                is_query: data_request.is_query,
                url: data_request.url.clone(),
            };

            Self::save_data_response_onchain(block_number, key, saved_request);
            Self::send_response_to_parachain(block_number, key)
        }

        #[pallet::weight(<T as Config>::WeightInfo::submit_data_via_api())]
        pub fn submit_data_via_api(
            origin: OriginFor<T>,
            para_id: Option<ParaId>,
            url: Vec<u8>,
            feed_name: Vec<u8>,
        ) -> DispatchResult {
            ensure_signed(origin.clone())?;
            let submitter_account_id = ensure_signed(origin.clone())?;
            let new_feed_name = (str::from_utf8(b"custom_").unwrap().to_owned()
                + str::from_utf8(&feed_name).unwrap())
            .as_bytes()
            .to_vec();
            let result = Self::ensure_account_owns_table(
                submitter_account_id.clone(),
                new_feed_name.clone(),
            );
            match result {
                Ok(()) => Self::add_data_request(
                    Some(submitter_account_id),
                    para_id,
                    Some(url),
                    new_feed_name,
                    Vec::new(),
                    false,
                ),
                _ => result,
            }
        }

        #[pallet::weight(<T as Config>::WeightInfo::submit_price_feed())]
        pub fn submit_price_feed(
            origin: OriginFor<T>,
            para_id: Option<ParaId>,
            requested_currencies: Vec<u8>,
        ) -> DispatchResult {
            let submitter_account_id = ensure_signed(origin.clone())?;
            let feed_name = "price_feeding".as_bytes().to_vec();
            let result =
                Self::ensure_account_owns_table(submitter_account_id.clone(), feed_name.clone());
            match result {
                Ok(()) => {
                    let currencies = str::from_utf8(&requested_currencies).unwrap();
                    let api_url =
                        str::from_utf8(b"https://api.kylin-node.co.uk/prices?currency_pairs=")
                            .unwrap();
                    let url = api_url.clone().to_owned() + currencies.clone();
                    Self::add_data_request(
                        Some(submitter_account_id),
                        para_id,
                        Some(url.as_bytes().to_vec()),
                        "price_feeding".as_bytes().to_vec(),
                        Vec::new(),
                        false,
                    )
                }
                _ => result,
            }
        }

        #[pallet::weight(<T as Config>::WeightInfo::sudo_remove_feed_account())]
        pub fn sudo_remove_feed_account(
            origin: OriginFor<T>,
            feed_name: Vec<u8>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let feed_exists = FeedAccountLookup::<T>::contains_key(feed_name.clone());
            if feed_exists {
                <FeedAccountLookup<T>>::remove(&feed_name);
                Self::deposit_event(Event::RemovedFeedAccount(feed_name.clone()))
            }
            Ok(())
        }

        #[pallet::weight(<T as Config>::WeightInfo::write_data_onchain())]
        pub fn write_data_onchain(
            origin: OriginFor<T>,
            feed_name: Vec<u8>,
            data: Vec<u8>,
        ) -> DispatchResult {
            ensure_signed(origin.clone())?;
            let submitter_account_id = ensure_signed(origin.clone())?;
            let new_feed_name = (str::from_utf8(b"custom_").unwrap().to_owned()
                + str::from_utf8(&feed_name).unwrap())
            .as_bytes()
            .to_vec();
            let result = Self::ensure_account_owns_table(
                submitter_account_id.clone(),
                new_feed_name.clone(),
            );
            match result {
                Ok(()) => Self::add_data_request(
                    Some(submitter_account_id),
                    None,
                    None,
                    new_feed_name,
                    data,
                    false,
                ),
                _ => result,
            }
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn xcm_submit_data_via_api(
            origin: OriginFor<T>,
            url: Vec<u8>,
            feed_name: Vec<u8>,
        ) -> DispatchResult {
            let requester_para_id =
                ensure_sibling_para(<T as Config>::Origin::from(origin.clone()))?;
            let submitter_account_id = ensure_signed(origin.clone())?;
            let new_feed_name = (str::from_utf8(b"custom_").unwrap().to_owned()
                + str::from_utf8(&feed_name).unwrap())
            .as_bytes()
            .to_vec();
            let result = Self::ensure_account_owns_table(
                submitter_account_id.clone(),
                new_feed_name.clone(),
            );
            match result {
                Ok(()) => Self::add_data_request(
                    Some(submitter_account_id),
                    Some(requester_para_id),
                    Some(url),
                    new_feed_name,
                    Vec::new(),
                    false,
                ),
                _ => result,
            }
        }

        /// Feed the external value.
		///
		/// Require authorized operator.
		#[pallet::weight(<T as Config>::WeightInfo::feed_data(values.len() as u32))]
		pub fn feed_data(
			origin: OriginFor<T>,
			values: Vec<(T::OracleKey, T::OracleValue)>,
		) -> DispatchResultWithPostInfo {
			let feeder = ensure_signed(origin.clone())?;
            // ensure feeder is authorized
            ensure!(T::Members::contains(&feeder), Error::<T>::NoPermission);
            // ensure account hasn't dispatched an updated yet
            ensure!(
                HasDispatched::<T>::mutate(|set| set.insert(feeder.clone())),
                Error::<T>::AlreadyFeeded
            );

            let now = T::Time::now();
            for (key, value) in &values {
                let timestamped = TimestampedValue {
                    value: value.clone(),
                    timestamp: now,
                };
                RawValues::<T>::insert(&feeder, &key, timestamped);

                // Update `Values` storage if `combined` yielded result.
                if let Some(combined) = Self::combined(key) {
                    <Values<T>>::insert(key, combined);
                }
            }

            Self::deposit_event(Event::NewFeedData { sender: feeder, values });
			Ok(Pays::No.into())
		}

    }

    // #[pallet::event where <T as frame_system::Config>:: AccountId: AsRef<[u8]> + ToHex + Decode + Serialize]
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    // #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config>
    where
        T::AccountId: AsRef<[u8]> + ToHex + Decode,
    {
        RemovedFeedAccount(Vec<u8>),
        SubmitNewData(
            Option<ParaId>,
            Vec<u8>,
            Option<Vec<u8>>,
            Option<T::AccountId>,
            T::BlockNumber,
        ),
        SavedToDWH(
            Option<ParaId>,
            Vec<u8>,
            Vec<u8>,
            DataRequest<ParaId, T::BlockNumber, T::AccountId>,
            T::BlockNumber,
        ),
        ReadFromDWH(
            Option<ParaId>,
            Vec<u8>,
            Vec<u8>,
            DataRequest<ParaId, T::BlockNumber, T::AccountId>,
            T::BlockNumber,
        ),
        ResponseSent(
            ParaId,
            DataRequest<ParaId, T::BlockNumber, T::AccountId>,
            T::BlockNumber,
        ),
        ErrorSendingResponse(
            SendError,
            ParaId,
            DataRequest<ParaId, T::BlockNumber, T::AccountId>,
        ),
        ResponseReceived(ParaId, Vec<u8>, Vec<u8>, T::BlockNumber),
        QueryFeeAwarded(
            T::AccountId,
            <<T as pallet::Config>::Currency as Currency<
                <T as frame_system::Config>::AccountId,
            >>::Balance,
            Vec<u8>,
        ),
        /// New feed data is submitted.
		NewFeedData {
			sender: T::AccountId,
			values: Vec<(T::OracleKey, T::OracleValue)>,
		},
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T>
    where
        T::AccountId: AsRef<[u8]> + ToHex + Decode,
        T: pallet_balances::Config,
    {
        type Call = Call<T>;

        /// Validate unsigned call to this module.
        ///
        /// By default unsigned transactions are disallowed, but implementing the validator
        /// here we make sure that some particular calls (the ones produced by offchain worker)
        /// are being whitelisted and marked as valid.
        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            if let Call::submit_data_unsigned {
                block_number,
                key: _,
                data: _,
            } = call
            {
                Self::validate_transaction(block_number)
            } else if let Call::clear_processed_requests_unsigned {
                block_number,
                processed_requests: _,
            } = call
            {
                Self::validate_transaction(block_number)
            } else if let Call::clear_api_queue_unsigned {
                block_number,
                processed_requests: _,
            } = call
            {
                Self::validate_transaction(block_number)
            } else {
                InvalidTransaction::Call.into()
            }
        }
    }

    #[pallet::type_value]
    pub fn InitialDataId<T: Config>() -> u64
    where
        <T as frame_system::Config>::AccountId: AsRef<[u8]> + ToHex
    {
        10000000u64
    }

    #[pallet::storage]
    pub type DataId<T: Config> = StorageValue<_, u64, ValueQuery, InitialDataId<T>>;

    #[pallet::storage]
    #[pallet::getter(fn data_requests)]
    pub type DataRequests<T: Config> =
        StorageMap<_, Identity, u64, DataRequest<ParaId, T::BlockNumber, T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn saved_data_requests)]
    pub type SavedRequests<T: Config> =
        StorageMap<_, Identity, u64, DataRequest<ParaId, T::BlockNumber, T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn api_queue)]
    pub type ApiQueue<T: Config> =
        StorageMap<_, Identity, u64, DataRequest<ParaId, T::BlockNumber, T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_unsigned_at)]
    pub(super) type NextUnsignedAt<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn feed_account_lookup)]
    pub(super) type FeedAccountLookup<T: Config> =
        StorageMap<_, Identity, Vec<u8>, (T::AccountId, Vec<u8>), OptionQuery>;

    type BufferIndex = u8;

    #[pallet::storage]
	#[pallet::getter(fn get_value)]
	pub(super) type BufferMap<T: Config> =
		StorageMap<_, Blake2_128Concat, BufferIndex, RingItem<ParaId, T::BlockNumber>, ValueQuery>;

	#[pallet::type_value]
	pub(super) fn BufferIndexDefaultValue() -> (BufferIndex, BufferIndex) {
		(0, 0)
	}

	#[pallet::storage]
	#[pallet::getter(fn range)]
	pub(super) type BufferRange<T: Config> =
		StorageValue<_, (BufferIndex, BufferIndex), ValueQuery, BufferIndexDefaultValue>;

    /// Raw values for each oracle operators
	#[pallet::storage]
	#[pallet::getter(fn raw_values)]
	pub type RawValues<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::OracleKey, TimestampedValueOf<T>>;

	/// Up to date combined value from Raw Values
	#[pallet::storage]
	#[pallet::getter(fn values)]
	pub type Values<T: Config> =
		StorageMap<_, Twox64Concat, <T as Config>::OracleKey, TimestampedValueOf<T>>;

	/// If an oracle operator has fed a value in this block
	#[pallet::storage]
	pub(crate) type HasDispatched<T: Config> =
		StorageValue<_, OrderedSet<T::AccountId, T::MaxHasDispatchedSize>, ValueQuery>;

}

#[derive(Clone, PartialEq, Eq, Default, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct DataRequest<ParaId, BlockNumber, AccountId> {
    para_id: Option<ParaId>,
    account_id: Option<AccountId>,
    requested_block_number: BlockNumber,
    processed_block_number: Option<BlockNumber>,
    requested_timestamp: u128,
    processed_timestamp: Option<u128>,

    payload: Vec<u8>,
    feed_name: Vec<u8>,
    is_query: bool,
    url: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RingItem<ParaId, BlockNumber> {
    para_id: Option<ParaId>,
    requested_block_number: BlockNumber,
    processed_block_number: Option<BlockNumber>,
    requested_timestamp: u128,
    processed_timestamp: Option<u128>,
    payload: Vec<u8>,
    feed_name: Vec<u8>,
    is_query: bool,
    url: Option<Vec<u8>>,
}

impl<BlockNumber, ParaId, AccountId> DataRequest<ParaId, BlockNumber, AccountId>
where 
    AccountId: AsRef<[u8]> + Clone, 
    BlockNumber: Clone + UniqueSaturatedInto<u32> + sp_std::fmt::Debug, 
    ParaId: Into<u32> + Copy
{
    fn to_json_string(&self, encoded_value: Vec<u8>) -> Vec<u8> {
        let mut object_elements = Vec::new();
        let para_key = str::from_utf8(b"para_id").unwrap().chars().collect();
        if self.para_id.is_some() {
            let para_id_number_value = NumberValue {
                integer: self.para_id.unwrap().into() as i64,
                fraction: 0,
                fraction_length: 0,
                exponent: 0,
            };
            object_elements.push((para_key, JsonValue::Number(para_id_number_value)));
        } else {
            object_elements.push((para_key, JsonValue::Null))
        }

        let account_id_key = str::from_utf8(b"account_id").unwrap().chars().collect();
        if self.account_id.is_some() {
            let account_id_in_hex = hex::encode(self.account_id.clone().unwrap().as_ref());
            object_elements.push((
                account_id_key,
                JsonValue::String(account_id_in_hex.chars().collect()),
            ));
        } else {
            object_elements.push((account_id_key, JsonValue::Null))
        }

        let requested_block_number_key = str::from_utf8(b"requested_block_number")
            .unwrap()
            .chars()
            .collect();
        let requested_block_number = NumberValue {
            integer: self.requested_block_number.clone().unique_saturated_into() as i64,
            fraction: 0,
            fraction_length: 0,
            exponent: 0,
        };
        object_elements.push((
            requested_block_number_key,
            JsonValue::Number(requested_block_number),
        ));

        let processed_block_number_key = str::from_utf8(b"processed_block_number")
            .unwrap()
            .chars()
            .collect();
        let processed_block_number = NumberValue {
            integer: self
                .processed_block_number
                .clone()
                .unwrap()
                .unique_saturated_into() as i64,
            fraction: 0,
            fraction_length: 0,
            exponent: 0,
        };
        object_elements.push((
            processed_block_number_key,
            JsonValue::Number(processed_block_number),
        ));

        let requested_timestamp_key = str::from_utf8(b"requested_timestamp")
            .unwrap()
            .chars()
            .collect();
        let requested_timestamp = NumberValue {
            integer: i64::try_from(self.requested_timestamp).unwrap(),
            fraction: 0,
            fraction_length: 0,
            exponent: 0,
        };
        object_elements.push((
            requested_timestamp_key,
            JsonValue::Number(requested_timestamp),
        ));

        let processed_timestamp_key = str::from_utf8(b"processed_timestamp")
            .unwrap()
            .chars()
            .collect();
        let processed_timestamp = NumberValue {
            integer: self.processed_timestamp.clone().unwrap() as i64,
            fraction: 0,
            fraction_length: 0,
            exponent: 0,
        };
        object_elements.push((
            processed_timestamp_key,
            JsonValue::Number(processed_timestamp),
        ));

        let payload_key = str::from_utf8(b"payload").unwrap().chars().collect();
        let payload = str::from_utf8(&self.payload).unwrap().chars().collect();
        object_elements.push((payload_key, JsonValue::String(payload)));

        let feed_name_key = str::from_utf8(b"feed_name").unwrap().chars().collect();
        let feed_name = str::from_utf8(&self.feed_name).unwrap().chars().collect();
        object_elements.push((feed_name_key, JsonValue::String(feed_name)));

        let url_key = str::from_utf8(b"url").unwrap().chars().collect();
        if self.url.is_some() {
            let url_string = self.url.as_ref().unwrap();
            let url = str::from_utf8(&url_string).unwrap().chars().collect();
            object_elements.push((url_key, JsonValue::String(url)));
        } else {
            object_elements.push((url_key, JsonValue::Null));
        }

        let json = JsonValue::Object(object_elements.clone());
        object_elements = Vec::new();
        let data_key = str::from_utf8(b"data").unwrap().chars().collect();
        object_elements.push((data_key, json));

        let hash_key = str::from_utf8(b"hash").unwrap().chars().collect();
        let encoded_hash =
            hex::encode(sp_runtime::traits::BlakeTwo256::hash(encoded_value.as_slice()).as_bytes())
                .chars()
                .collect();
        object_elements.push((hash_key, JsonValue::String(encoded_hash)));

        let final_json = JsonValue::Object(object_elements.clone()).format(4);

        let json_output = str::from_utf8(&final_json).unwrap().as_bytes().to_vec();
        return json_output;
    }
}
enum TransactionType {
    Signed,
    UnsignedForAny,
    UnsignedForAll,
    Raw,
    None,
}

impl<T: Config> Pallet<T>
where T::AccountId: AsRef<[u8]>
{
    fn ensure_account_owns_table(
        submitter_account_id: T::AccountId,
        feed_name: Vec<u8>,
    ) -> DispatchResult {
        let feed_exists = FeedAccountLookup::<T>::contains_key(feed_name.clone());
        if feed_exists {
            let feed_owner = Self::feed_account_lookup(feed_name).unwrap().0;
            if feed_owner == submitter_account_id {
                Ok(())
            } else {
                Err(DispatchError::BadOrigin)
            }
        } else {
            let new_hash: Vec<u8> = Vec::new();
            <FeedAccountLookup<T>>::insert(feed_name, (submitter_account_id.clone(), new_hash));
            Ok(())
        }
    }

    fn query_feed(
        submitter_account_id: T::AccountId,
        para_id: Option<ParaId>,
        feed_name: Vec<u8>,
    ) -> DispatchResult {
        let feed_exists = FeedAccountLookup::<T>::contains_key(feed_name.clone());
        if feed_exists {
            let feed = Self::feed_account_lookup(feed_name.clone()).unwrap();
            let latest_hash = feed.1;
            let api_url = str::from_utf8(b"https://api.kylin-node.co.uk/query?hash=").unwrap();
            let url = api_url.clone().to_owned() + str::from_utf8(&latest_hash.clone()).unwrap();

            let total_reward = {
                let call = Call::query_data {
                    para_id: para_id.clone(),
                    feed_name: feed_name.clone(),
                };
                let call_fee = T::EstimateCallFee::estimate_call_fee(&call, None.into());
                call_fee
            };
            let query_fee = total_reward * 25u32.into() / 100u32.into();
            T::Currency::deposit_into_existing(&feed.0, query_fee)?;
            Self::deposit_event(Event::QueryFeeAwarded(feed.0, query_fee, feed_name.clone()));

            Self::add_data_request(
                Some(submitter_account_id),
                para_id,
                Some(url.as_bytes().to_vec()),
                feed_name.clone(),
                Vec::new(),
                true,
            )
        } else {
            Err(DispatchError::CannotLookup)
        }
    }

    fn choose_transaction_type(block_number: T::BlockNumber) -> TransactionType {
        /// A friendlier name for the error that is going to be returned in case we are in the grace
        /// period.
        const RECENTLY_SENT: () = ();

        // Start off by creating a reference to Local Storage value.
        // Since the local storage is common for all offchain workers, it's a good practice
        // to prepend your entry with the module name.
        let val = StorageValueRef::persistent(b"kylin_oracle::last_send");
        // The Local Storage is persisted and shared between runs of the offchain workers,
        // and offchain workers may run concurrently. We can use the `mutate` function, to
        // write a storage entry in an atomic fashion. Under the hood it uses `compare_and_set`
        // low-level method of local storage API, which means that only one worker
        // will be able to "acquire a lock" and send a transaction if multiple workers
        // happen to be executed concurrently.
        let res = val.mutate(
            |last_send: Result<Option<T::BlockNumber>, StorageRetrievalError>| {
                match last_send {
                    // If we already have a value in storage and the block number is recent enough
                    // we avoid sending another transaction at this time.
                    Ok(Some(block)) if block_number < block => Err(RECENTLY_SENT),
                    // In every other case we attempt to acquire the lock and send a transaction.
                    _ => Ok(block_number),
                }
            },
        );

        // The result of `mutate` call will give us a nested `Result` type.
        // The first one matches the return of the closure passed to `mutate`, i.e.
        // if we return `Err` from the closure, we get an `Err` here.
        // In case we return `Ok`, here we will have another (inner) `Result` that indicates
        // if the value has been set to the storage correctly - i.e. if it wasn't
        // written to in the meantime.
        match res {
            // The value has been set correctly, which means we can safely send a transaction now.
            Ok(block_number) => {
                // Depending if the block is even or odd we will send a `Signed` or `Unsigned`
                // transaction.
                // Note that this logic doesn't really guarantee that the transactions will be sent
                // in an alternating fashion (i.e. fairly distributed). Depending on the execution
                // order and lock acquisition, we may end up for instance sending two `Signed`
                // transactions in a row. If a strict order is desired, it's better to use
                // the storage entry for that. (for instance store both block number and a flag
                // indicating the type of next transaction to send).
                let transaction_type = block_number % 3u32.into();
                if transaction_type == Zero::zero() {
                    TransactionType::Signed
                } else if transaction_type == T::BlockNumber::from(1u32) {
                    TransactionType::UnsignedForAny
                } else if transaction_type == T::BlockNumber::from(2u32) {
                    TransactionType::UnsignedForAll
                } else {
                    TransactionType::Raw
                }
            }
            // We are in the grace period, we should not send a transaction this time.
            Err(MutateStorageError::ValueFunctionFailed(RECENTLY_SENT)) => TransactionType::None,
            // We wanted to send a transaction, but failed to write the block number (acquire a
            // lock). This indicates that another offchain worker that was running concurrently
            // most likely executed the same logic and succeeded at writing to storage.
            // Thus we don't really want to send the transaction, knowing that the other run
            // already did.
            Err(MutateStorageError::ConcurrentModification(_)) => TransactionType::None,
        }
    }

    fn add_data_request(
        account_id: Option<T::AccountId>,
        para_id: Option<ParaId>,
        url: Option<Vec<u8>>,
        feed_name: Vec<u8>,
        payload: Vec<u8>,
        is_query: bool,
    ) -> DispatchResult {
        let index = DataId::<T>::get();
        let block_number = <system::Pallet<T>>::block_number();
        let current_timestamp = T::UnixTime::now().as_millis();

        <DataRequests<T>>::insert(
            index,
            DataRequest {
                para_id: para_id,
                account_id: account_id.clone(),
                feed_name: feed_name.clone(),
                requested_block_number: block_number,
                processed_block_number: None,
                requested_timestamp: current_timestamp,
                processed_timestamp: None,
                payload: payload,
                is_query: is_query,
                url: url.clone(),
            },
        );

        if !is_query {
            Self::deposit_event(Event::SubmitNewData(
                para_id,
                feed_name.clone(),
                url.clone(),
                account_id.clone(),
                block_number,
            ));
        }

        DataId::<T>::put(index + 1u64);
        Ok(())
    }

    fn save_data_response_onchain(
        block_number: T::BlockNumber,
        key: u64,
        data_request: DataRequest<ParaId, T::BlockNumber, T::AccountId>,
    ) -> () {
        let current_timestamp = T::UnixTime::now().as_millis();
        // let saved_data_request = DataRequest {
        //     para_id: data_request.para_id.clone(),
        //     account_id: data_request.account_id.clone(),
        //     feed_name: data_request.feed_name.clone(),
        //     requested_block_number: data_request.requested_block_number,
        //     processed_block_number: Some(block_number),
        //     requested_timestamp: data_request.requested_timestamp,
        //     processed_timestamp: Some(current_timestamp),
        //     payload: data_request.payload.clone(),
        //     is_query: data_request.is_query,
        //     url: data_request.url.clone(),
        // };
        //<SavedRequests<T>>::insert(key, saved_data_request.clone());

        let saved_item = RingItem {
            para_id: data_request.para_id.clone(),
            feed_name: data_request.feed_name.clone(),
            requested_block_number: data_request.requested_block_number,
            processed_block_number: Some(block_number),
            requested_timestamp: data_request.requested_timestamp,
            processed_timestamp: Some(current_timestamp),
            payload: data_request.payload.clone(),
            is_query: data_request.is_query,
            url: data_request.url.clone(),
        };
        
        let mut queue = Self::queue_transient();
        queue.push(saved_item.clone());
    }

    fn send_response_to_parachain(block_number: T::BlockNumber, key: u64) -> DispatchResult {
        let saved_request = Self::saved_data_requests(key).unwrap();
        if saved_request.para_id.is_some() {
            match T::XcmSender::send_xcm(
                (
                    1,
                    Junction::Parachain(saved_request.para_id.unwrap().into()),
                ),
                Xcm(vec![Transact {
                    origin_type: OriginKind::Native,
                    require_weight_at_most: 1_000,
                    call: <T as Config>::Call::from(Call::<T>::receive_response_from_parachain {
                        feed_name: saved_request.feed_name.clone(),
                        response: saved_request.payload.clone(),
                    })
                    .encode()
                    .into(),
                }]),
            ) {
                Ok(()) => Self::deposit_event(Event::ResponseSent(
                    saved_request.para_id.unwrap(),
                    saved_request.clone(),
                    block_number,
                )),
                Err(e) => Self::deposit_event(Event::ErrorSendingResponse(
                    e,
                    saved_request.para_id.unwrap(),
                    saved_request.clone(),
                )),
            }
        }
        Ok(())
    }

    /// A helper function to fetch the price and send signed transaction.
    fn fetch_data_and_send_signed(block_number: T::BlockNumber) -> Result<(), &'static str> {
        let signer = Signer::<T, T::AuthorityId>::all_accounts();
        if !signer.can_sign() {
            return Err(
                "No local accounts available. Consider adding one via `author_insertKey` RPC.",
            )?;
        }
        let mut processed_requests: Vec<u64> = Vec::new();

        for (key, val) in <DataRequests<T> as IterableStorageMap<_, _>>::iter() {
            let mut response = val.payload.clone();
            if val.url.is_some() {
                response = Self::fetch_http_get_result(val.url.clone().unwrap())
                    .unwrap_or("Failed fetch data".as_bytes().to_vec());
            };

            // write data to chain
            processed_requests.push(key);
            let results = signer.send_signed_transaction(|_account| Call::submit_data_signed {
                block_number: block_number,
                key: key,
                data: response.clone(),
            });
            for (acc, res) in &results {
                match res {
                    Ok(()) => log::info!("[{:?}] Submitted data {}", acc.id, key),
                    Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
                }
            }
        }
        if processed_requests.iter().count() > 0 {
            let results = signer.send_signed_transaction(|_account| {
                Call::clear_processed_requests_unsigned {
                    block_number: block_number,
                    processed_requests: processed_requests.clone(),
                }
            });
            for (acc, res) in &results {
                match res {
                    Ok(()) => log::info!("[{:?}] Clearing out processed requests.", acc.id),
                    Err(e) => log::error!(
                        "[{:?}] Failed to clear out processed requests: {:?}",
                        acc.id,
                        e
                    ),
                }
            }
        }

        let mut queue_to_api: Vec<u64> = Vec::new();
        for (key, val) in <ApiQueue<T> as IterableStorageMap<_, _>>::iter() {
            // write data to postgres dB
            let url = str::from_utf8(b"https://api.kylin-node.co.uk/submit").unwrap();
            let _post_response = Self::submit_http_post_request(url.as_bytes().to_vec(), val)
                .unwrap_or("Failed to submit data".as_bytes().to_vec());
            queue_to_api.push(key);
        }
        if queue_to_api.iter().count() > 0 {
            let result = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(
                Call::clear_api_queue_unsigned {
                    block_number: block_number,
                    processed_requests: queue_to_api,
                }
                .into(),
            );
            if let Err(e) = result {
                log::error!("Error clearing api queue: {:?}", e);
            }
        }
        Ok(())
    }

    fn fetch_data_and_send_raw_unsigned(block_number: T::BlockNumber) -> Result<(), &'static str> {
        let next_unsigned_at = <NextUnsignedAt<T>>::get();
        if next_unsigned_at > block_number {
            return Err("Too early to send unsigned transaction");
        }

        let mut processed_requests: Vec<u64> = Vec::new();
        for (key, val) in <DataRequests<T> as IterableStorageMap<_, _>>::iter() {
            let mut response = val.payload.clone();
            if val.url.is_some() {
                response = Self::fetch_http_get_result(val.url.clone().unwrap())
                    .unwrap_or("Failed fetch data".as_bytes().to_vec());
            }

            processed_requests.push(key);
            let result = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(
                Call::submit_data_unsigned {
                    block_number: block_number,
                    key: key,
                    data: response,
                }
                .into(),
            );

            if let Err(e) = result {
                log::error!("Error submitting unsigned transaction: {:?}", e);
            }
        }
        if processed_requests.iter().count() > 0 {
            let result = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(
                Call::clear_processed_requests_unsigned {
                    block_number: block_number,
                    processed_requests: processed_requests,
                }
                .into(),
            );

            if let Err(e) = result {
                log::error!("Error clearing queue: {:?}", e);
            }
        }

        let mut queue_to_api: Vec<u64> = Vec::new();

        for (key, val) in <ApiQueue<T> as IterableStorageMap<_, _>>::iter() {
            // write data to postgres dB
            let url = str::from_utf8(b"https://api.kylin-node.co.uk/submit").unwrap();
            let _post_response = Self::submit_http_post_request(url.as_bytes().to_vec(), val)
                .unwrap_or("Failed to submit data".as_bytes().to_vec());
            queue_to_api.push(key);
        }

        if queue_to_api.iter().count() > 0 {
            let result = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(
                Call::clear_api_queue_unsigned {
                    block_number: block_number,
                    processed_requests: queue_to_api,
                }
                .into(),
            );
            if let Err(e) = result {
                log::error!("Error clearing api queue: {:?}", e);
            }
        }

        Ok(())
    }

    /// Fetch current price and return the result in cents.
    fn fetch_http_get_result(url: Vec<u8>) -> Result<Vec<u8>, http::Error> {
        // We want to keep the offchain worker execution time reasonable, so we set a hard-coded
        // deadline to 2s to complete the external call.
        // You can also wait idefinitely for the response, however you may still get a timeout
        // coming from the host machine.
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(10_000));
        // Initiate an external HTTP GET request.
        // This is using high-level wrappers from `sp_runtime`, for the low-level calls that
        // you can find in `sp_io`. The API is trying to be similar to `reqwest`, but
        // since we are running in a custom WASM execution environment we can't simply
        // import the library here.
        let request = http::Request::get(str::from_utf8(&url).unwrap());

        // We set the deadline for sending of the request, note that awaiting response can§
        // have a separate deadline. Next we send the request, before that it's also possible
        // to alter request headers or stream body content in case of non-GET requests.
        let pending = request
            .deadline(deadline)
            .send()
            .map_err(|_| http::Error::IoError)?;

        // The request is already being processed by the host, we are free to do anything
        // else in the worker (we can send multiple concurrent requests too).
        // At some point however we probably want to check the response though,
        // so we can block current thread and wait for it to finish.
        // Note that since the request is being driven by the host, we don't have to wait
        // for the request to have it complete, we will just not read the response.
        let response = pending
            .try_wait(deadline)
            .map_err(|_| http::Error::DeadlineReached)??;

        // Let's check the status code before we proceed to reading the response.
        if response.code != 200 {
            log::info!("Unexpected status code: {}", response.code);
            return Err(http::Error::Unknown);
        }

        // Next we want to fully read the response body and collect it to a vector of bytes.
        // Note that the return object allows you to read the body in chunks as well
        // with a way to control the deadline.
        let body = response.body().collect::<Vec<u8>>();
        // Create a str slice from the body.
        let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
            log::info!("No UTF8 body");
            http::Error::Unknown
        })?;

        Ok(body_str.clone().as_bytes().to_vec())
    }

    fn submit_http_post_request(
        url: Vec<u8>,
        val: DataRequest<ParaId, T::BlockNumber, T::AccountId>,
    ) -> Result<Vec<u8>, http::Error> {
        // Establish deadline
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(10_000));
        let encoded_hash = val.clone().encode();

        let request_body = val.clone().to_json_string(encoded_hash.clone());
        let request =
            http::Request::post(str::from_utf8(&url).unwrap(), vec![request_body.clone()])
                .add_header("x-api-key", "test_api_key")
                .add_header("content-type", "application/json");

        // Send post request
        let pending = request
            .deadline(deadline)
            .body(vec![request_body.clone()])
            .send()
            .map_err(|_| http::Error::IoError)?;

        // Wait for response
        let response = pending
            .try_wait(deadline)
            .map_err(|_| http::Error::DeadlineReached)??;

        // Check status code
        if response.code != 200 {
            log::info!("Unexpected status code: {}", response.code);
            return Err(http::Error::Unknown);
        }

        // Collect body
        let body = response.body().collect::<Vec<u8>>();
        let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
            log::info!("No UTF8 body");
            http::Error::Unknown
        })?;

        Ok(body_str.as_bytes().to_vec())
    }

    fn validate_transaction(block_number: &T::BlockNumber) -> TransactionValidity {
        // Now let's check if the transaction has any chance to succeed.
        let next_unsigned_at = <NextUnsignedAt<T>>::get();
        if &next_unsigned_at > block_number {
            return InvalidTransaction::Stale.into();
        }
        // Let's make sure to reject transactions from the future.
        let current_block = <system::Pallet<T>>::block_number();
        if &current_block < block_number {
            return InvalidTransaction::Future.into();
        }
        ValidTransaction::with_tag_prefix("KylinOCW")
            .priority(T::UnsignedPriority::get())
            .longevity(5)
            .propagate(true)
            .build()
    }

    fn queue_transient() -> Box<dyn RingBufferTrait<RingItem<ParaId, T::BlockNumber>>> {
		Box::new(RingBufferTransient::<
			RingItem<ParaId, T::BlockNumber>,
			<Self as Store>::BufferRange,
			<Self as Store>::BufferMap,
			u8,
		>::new())
	}

    pub fn read_raw_values(key: &T::OracleKey) -> Vec<TimestampedValueOf<T>> {
		T::Members::sorted_members()
			.iter()
			.filter_map(|x| Self::raw_values(x, key))
			.collect()
	}

	/// Fetch current combined value.
	pub fn get(key: &T::OracleKey) -> Option<TimestampedValueOf<T>> {
		Self::values(key)
	}

	#[allow(clippy::complexity)]
	pub fn get_all_values() -> Vec<(T::OracleKey, Option<TimestampedValueOf<T>>)> {
		<Values<T>>::iter().map(|(k, v)| (k, Some(v))).collect()
	}

	fn combined(key: &T::OracleKey) -> Option<TimestampedValueOf<T>> {
		let values = Self::read_raw_values(key);
		T::CombineData::combine_data(key, values, Self::values(key))
	}
}
