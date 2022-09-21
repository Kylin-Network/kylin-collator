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

        /// Provide the implementation to combine raw values to produce
		/// aggregated value
		type CombineData: CombineData<Self::OracleKey, TimestampedValueOf<Self>>;

        /// The data key type
		type OracleKey: Parameter + Member + Eq + Into<Vec<u8>>;

		/// The data value type
		type OracleValue: Parameter + Member + Ord + From<i64>;

        /// Oracle operators.
		type Members: SortedMembers<Self::AccountId>;

		/// Maximum size of HasDispatched
		#[pallet::constant]
		type MaxHasDispatchedSize: Get<u32>;

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
            let res = Self::fetch_api_and_feed_data(block_number);

            // let should_send = Self::choose_transaction_type(block_number);
            // let res = match should_send {
            //     TransactionType::Signed => Self::fetch_data_and_send_signed(block_number),
            //     TransactionType::Raw
            //     | TransactionType::UnsignedForAll
            //     | TransactionType::UnsignedForAny => {
            //         Self::fetch_data_and_send_raw_unsigned(block_number)
            //     }
            //     _ => Ok(()),
            // };
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

            let now = T::UnixTime::now().as_millis();
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

        #[pallet::weight(<T as Config>::WeightInfo::feed_data(values.len() as u32))]
		pub fn xcm_feed_data(
			origin: OriginFor<T>,
			values: Vec<(T::OracleKey, T::OracleValue)>,
		) -> DispatchResultWithPostInfo {
            let para_id =
                ensure_sibling_para(<T as Config>::Origin::from(origin.clone()))?;

            // // ensure feeder is authorized
            // ensure!(T::Members::contains(&feeder), Error::<T>::NoPermission);
            // // ensure account hasn't dispatched an updated yet
            // ensure!(
            //     HasDispatched::<T>::mutate(|set| set.insert(feeder.clone())),
            //     Error::<T>::AlreadyFeeded
            // );

            let now = T::UnixTime::now().as_millis();
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

            Self::deposit_event(Event::NewParaFeedData {para_id, values });
			Ok(Pays::No.into())
		}

        //#[pallet::weight(T::BlockWeights::get().max_block)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn xcm_evt(
			origin: OriginFor<T>,
		) -> DispatchResultWithPostInfo {
            let para_id =
                ensure_sibling_para(<T as Config>::Origin::from(origin.clone()))?;

            Self::deposit_event(Event::NewParaEvt {para_id });
			Ok(Pays::No.into())
		}

        //#[pallet::weight(T::BlockWeights::get().max_block)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn xcm_evt1(
			origin: OriginFor<T>,
		) -> DispatchResultWithPostInfo {
            let feeder = ensure_signed(origin.clone())?;

            Self::deposit_event(Event::NewParaEvt1 {sender: feeder });
			Ok(Pays::No.into())
		}

        #[pallet::weight(<T as Config>::WeightInfo::submit_api())]
        pub fn submit_api(
            origin: OriginFor<T>,
            key: T::OracleKey,
            url: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let submitter = ensure_signed(origin.clone())?;
            // ensure submitter is authorized
            ensure!(T::Members::contains(&submitter), Error::<T>::NoPermission);

            let block_number = <system::Pallet<T>>::block_number();
            let feed = ApiFeed {
                    requested_block_number: block_number,
                    url: Some(url),
                };
            ApiFeeds::<T>::insert(&submitter, &key, feed.clone());

            Self::deposit_event(Event::NewApiFeed { sender: submitter, key, feed });
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
                Self::deposit_event(Event::ApiFeedRemoved { sender: submitter, key, feed });
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
        FeedDataSent(
            ParaId,
        ),
        FeedDataError(
            SendError,
            ParaId,
        ),
        /// New feed data is submitted.
		NewFeedData {
			sender: T::AccountId,
			values: Vec<(T::OracleKey, T::OracleValue)>,
		},
        /// New feed data is submitted.
		NewParaFeedData {
            para_id: ParaId,
			values: Vec<(T::OracleKey, T::OracleValue)>,
		},
        NewParaEvt {
            para_id: ParaId,
		},
        NewParaEvt1 {
            sender: T::AccountId,
		},
        /// New feed is submitted.
		NewApiFeed {
			sender: T::AccountId,
            key: T::OracleKey,
            feed: ApiFeed<T::BlockNumber>,
		},
        /// Apifeed is removed.
		ApiFeedRemoved {
			sender: T::AccountId,
            key: T::OracleKey,
            feed: ApiFeed<T::BlockNumber>,
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
            InvalidTransaction::Call.into()
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn next_unsigned_at)]
    pub(super) type NextUnsignedAt<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    #[pallet::storage]
	#[pallet::getter(fn api_feeds)]
	pub type ApiFeeds<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::OracleKey, ApiFeed<T::BlockNumber>>;

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

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ApiFeed<BlockNumber> {
    requested_block_number: BlockNumber,
    url: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ParaFeed<ParaId, BlockNumber> {
    requested_block_number: BlockNumber,
    para_id: Option<ParaId>,
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

    /// A helper function to fetch the price and send signed transaction.
    fn fetch_api_and_feed_data(block_number: T::BlockNumber) -> Result<(), &'static str> {
        let signer = Signer::<T, T::AuthorityId>::all_accounts();
        if !signer.can_sign() {
            return Err(
                "No local accounts available. Consider adding one via `author_insertKey` RPC.",
            )?;
        }

        let mut values = Vec::<(T::OracleKey, T::OracleValue)>::new();
        for (_acc, key, val) in <ApiFeeds<T> as IterableStorageDoubleMap<_, _, _>>::iter() {
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

	fn combined(key: &T::OracleKey) -> Option<TimestampedValueOf<T>> {
		let values = Self::read_raw_values(key);
		T::CombineData::combine_data(key, values, Self::values(key))
	}
}
