#![cfg_attr(not(feature = "std"), no_std)]
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};
use cumulus_pallet_xcm::{ensure_sibling_para, Origin as CumulusOrigin};
use cumulus_primitives_core::ParaId;
use frame_support::{
    dispatch::DispatchResultWithPostInfo,
    log,
    pallet_prelude::*,
    traits::{Currency, EstimateCallFee, UnixTime, ChangeMembers, Get, SortedMembers},
    IterableStorageMap, IterableStorageDoubleMap,
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
//use num_traits::float::Float;

pub use pallet::*;
#[cfg(test)]
mod tests;

mod default_combine_data;
pub use default_combine_data::DefaultCombineData;

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

    //pub(crate) type MomentOf<T> = <<T as Config>::Time as Time>::Moment;
	pub(crate) type TimestampedValueOf<T> = TimestampedValue<<T as Config>::OracleValue, u128>;

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

        /// The data key type
		type OracleKey: Parameter + Member + Eq + Into<Vec<u8>>;

		/// The data value type
		type OracleValue: Parameter + Member + Ord + From<i64>;

        /// Oracle operators.
		type Members: SortedMembers<Self::AccountId>;


    }

    #[pallet::pallet]
    #[pallet::generate_store(trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter( fn running_status)]
    type SystemRunnig<T> = StorageValue<_, bool, ValueQuery>;


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
            
            let res = Self::fetch_api_and_feed_data(block_number);
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
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn submit_data(
            origin: OriginFor<T>,
            para_id: ParaId,
            values: Vec<(T::OracleKey, T::OracleValue)>,
        ) -> DispatchResult {
            feed_data_to_parachain(ParaId, values)
        }

        #[pallet::weight(<T as Config>::WeightInfo::submit_api())]
        pub fn submit_api(
            origin: OriginFor<T>,
            para_id: Option<ParaId>,
            key: T::OracleKey,
            url: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let submitter = ensure_signed(origin.clone())?;
            // ensure submitter is authorized
            ensure!(T::Members::contains(&submitter), Error::<T>::NoPermission);

            let block_number = <system::Pallet<T>>::block_number();
            let feed = ApiFeed {
                    requested_block_number: block_number,
                    para_id: para_id,
                    url: Some(url),
                };
            ApiFeeds::<T>::insert(&submitter, &key, feed.clone());

            Self::deposit_event(Event::NewFeed { sender: submitter, key, feed });
			Ok(Pays::No.into())
        }

        #[pallet::weight(<T as Config>::WeightInfo::clear_api())]
        pub fn clear_api(
            origin: OriginFor<T>,
            key: T::OracleKey,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin.clone())?;
            // ensure submitter is authorized
            ensure!(T::Members::contains(&submitter), Error::<T>::NoPermission);

            let feed_exists = ApiFeeds::<T>::contains_key(&submitter, &key);
            if feed_exists {
                let feed = Self::api_feeds(&submitter, &key).unwrap();
                <ApiFeeds<T>>::remove(&submitter, &key);
                Self::deposit_event(Event::FeedRemoved { sender: submitter, key, feed });
                Ok(())
            } else {
                Err(DispatchError::CannotLookup)
            }
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
        /// New feed is submitted.
		NewFeed {
			sender: T::AccountId,
            key: T::OracleKey,
            feed: ApiFeed<ParaId, T::BlockNumber>,
		},
        /// Feed is removed.
		FeedRemoved {
			sender: T::AccountId,
            key: T::OracleKey,
            feed: ApiFeed<ParaId, T::BlockNumber>,
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

    #[pallet::storage]
	#[pallet::getter(fn api_feeds)]
	pub type ApiFeeds<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::OracleKey, ApiFeed<ParaId, T::BlockNumber>>;

}


#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ApiFeed<ParaId, BlockNumber> {
    requested_block_number: BlockNumber,
    para_id: Option<ParaId>,
    url: Option<Vec<u8>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct CryptoComparePrice {
    pub usdt: f64,
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
    /// A helper function to fetch the price and send signed transaction.
    fn fetch_api_and_feed_data(block_number: T::BlockNumber) -> Result<(), &'static str> {
        let signer = Signer::<T, T::AuthorityId>::all_accounts();
        if !signer.can_sign() {
            return Err(
                "No local accounts available. Consider adding one via `author_insertKey` RPC.",
            )?;
        }

        let mut values = Vec::<(T::OracleKey, T::OracleValue)>::new();
        for (acc, key, val) in <ApiFeeds<T> as IterableStorageDoubleMap<_, _, _>>::iter() {
            // let mut response :Vec<u8>;
            if val.url.is_some() {
                let response = Self::fetch_http_get_result(val.url.clone().unwrap())
                    .unwrap_or("Failed fetch data".as_bytes().to_vec());

                let oval :T::OracleValue;
                match str::from_utf8(&key.clone().into()) {
                    Ok("CCApi") => {
                        let price: CryptoComparePrice = serde_json::from_slice(&response)
                            .expect("Response JSON was not well-formatted");
                        // We only store int, so every float will be convert to int with 6 decimals pad
                        let pval = (price.usdt * 1000000.0) as i64;
                        oval = pval.into();
                        values.push((key.clone(), oval));
                    },
                    Ok("CWApi") => {
                        let price: CryptoComparePrice = serde_json::from_slice(&response)
                            .expect("Response JSON was not well-formatted");
                        // We only store int, so every float will be convert to int with 6 decimals pad
                        let pval = (price.usdt * 1000000.0) as i64;
                        oval = pval.into();
                        values.push((key.clone(), oval));
                    },
                    _ => (),
                }
                
            };
        }

        if values.iter().count() > 0 {
            // write data to chain
            let results = signer.send_signed_transaction(|_account| Call::feed_data {
                values: values.clone(),
            });
            for (acc, res) in &results {
                match res {
                    Ok(()) => log::info!("[{:?}] Submitted data", acc.id),
                    Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
                }
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

    fn feed_data_to_parachain(para_id: ParaId, values: Vec<(T::OracleKey, T::OracleValue)>) -> DispatchResult {
        let saved_request = Self::saved_data_requests(key).unwrap();
        if saved_request.para_id.is_some() {

            match T::XcmSender::send_xcm(
                (
                    1,
                    Junction::Parachain(para_id.into()),
                ),
                Xcm(vec![Transact {
                    origin_type: OriginKind::Native,
                    require_weight_at_most: 1_000,
                    call: <T as Config>::Call::from(Call::<T>::xcm_feed_data {
                        values: values.clone(),
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


}
