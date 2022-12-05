#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};
use cumulus_pallet_xcm::{ensure_sibling_para, Origin as CumulusOrigin};
use cumulus_primitives_core::ParaId;
use frame_support::{
    dispatch::{GetDispatchInfo, DispatchResultWithPostInfo},
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

pub use pallet::*;
#[cfg(test)]
mod tests;

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
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocrp");
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

/// Mock structure for XCM Call message encoding
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[allow(non_camel_case_types)]
enum KylinMockFunc {
    #[codec(index = 1u8)]
    xcm_feed_data { values: Vec<(Vec<u8>, i64)> },
    #[codec(index = 2u8)]
    xcm_query_data { key: Vec<u8> },
}

/// Mock structure for XCM Call message encoding
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[allow(non_camel_case_types)]
enum KylinMockCall {
    #[codec(index = 166u8)]
    KylinOraclePallet(KylinMockFunc),
}

/// An index to a block.
///
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config + pallet_balances::Config
    where <Self as frame_system::Config>::AccountId: AsRef<[u8]> + ToHex
    {
        /// The identifier type for an offchain worker.
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

        type RuntimeOrigin: From<<Self as SystemConfig>::RuntimeOrigin>
            + Into<Result<CumulusOrigin, <Self as Config>::RuntimeOrigin>>;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The overarching dispatch call type.
        type RuntimeCall: From<Call<Self>> + Encode;

        type XcmSender: SendXcm;

        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;

        type Currency: frame_support::traits::Currency<Self::AccountId>;

        /// Oracle operators.
		type Members: SortedMembers<Self::AccountId>;

    }

    #[pallet::pallet]
    #[pallet::generate_store(trait Store)]
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
        /// Set the parachain ID of the Kylin Oracle chain.
		///
		/// Can be called by authorized origin.
		///
		/// # Parameter:
		/// * `para_id` - parachain ID of the Kylin Oracle chain
		/// 
        #[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
        pub fn set_kylin_id(
            origin: OriginFor<T>,
            para_id: ParaId,
        ) -> DispatchResult {
            // SBP-M1 review: everyone can modify KylinParaId
            <KylinParaId<T>>::put(para_id);
            Ok(())
        }

        /// Feed the external value.
		///
		/// Call by the offchain worker.
		///
		/// # Parameter:
        /// * `para_id` - parachain ID of the Oracle chain
		/// * `values` - value array for the feed
		/// 
        #[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
        pub fn feed_data(
            origin: OriginFor<T>,
            para_id: ParaId,
            values: Vec<(Vec<u8>, i64)>,
        ) -> DispatchResult {
            // SBP-M1 review: can be called by anyone
            // You should introduce signature verification
            Self::feed_data_to_parachain(para_id, values)
        }

        /// Submit the URL Endpoint for the feed.
		///
		/// Can be called by authorized origin.
		///
		/// # Parameter:
		/// * `key` - key for the feed
		/// * `url` - url for the feed
		/// 
		/// # Emits
		/// * `NewFeed`
        #[pallet::weight(<T as Config>::WeightInfo::submit_api())]
        pub fn submit_api(
            origin: OriginFor<T>,
            // SBP-M1 review: security issue -> unbounded vectors
            key: Vec<u8>,
            url: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            // SBP-M1 review: do not clone if unnecessary
            // See https://github.com/paritytech/substrate/blob/polkadot-v0.9.31/frame/assets/src/lib.rs#L693
            let submitter = ensure_signed(origin.clone())?;
            // SBP-M1 review: remove the code when is not useful
            // ensure submitter is authorized
            //ensure!(T::Members::contains(&submitter), Error::<T>::NoPermission);

            let block_number = <system::Pallet<T>>::block_number();
            let feed = ApiFeed {
                    requested_block_number: block_number,
                    url: Some(url),
                };
            ApiFeeds::<T>::insert(&submitter, &key, feed.clone());

            Self::deposit_event(Event::NewFeed { sender: submitter, key, feed });
			Ok(Pays::No.into())
        }

        /// Remove the URL Endpoint for the feed.
		///
		/// Can be called by authorized origin.
		///
		/// # Parameter:
		/// * `key` - key for the feed
		/// 
		/// # Emits
		/// * `FeedRemoved`
        #[pallet::weight(<T as Config>::WeightInfo::remove_api())]
        pub fn remove_api(
            origin: OriginFor<T>,
            // SBP-M1 review: unbounded vector...
            key: Vec<u8>,
        ) -> DispatchResult {
            // SBP-M1 review: do not clone if unnecessary
            // See https://github.com/paritytech/substrate/blob/polkadot-v0.9.31/frame/assets/src/lib.rs#L693
            let submitter = ensure_signed(origin.clone())?;
            // ensure submitter is authorized
            //ensure!(T::Members::contains(&submitter), Error::<T>::NoPermission);

            let feed_exists = ApiFeeds::<T>::contains_key(&submitter, &key);
            if feed_exists {
                // SBP-M1 review: use proper error handling instead of unwrap
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
        /// Feed data have been sent.
        FeedDataSent(
            ParaId,
        ),
        /// Feed data sending error.
        FeedDataError(
            SendError,
            ParaId,
        ),
        /// New feed is submitted.
		NewFeed {
			sender: T::AccountId,
            key: Vec<u8>,
            feed: ApiFeed<T::BlockNumber>,
		},
        /// Feed is removed.
		FeedRemoved {
			sender: T::AccountId,
            key: Vec<u8>,
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
            // SBP-M1 review: as I understand properly 
            // every unsigned tx will be invalid
            // So why have you implemented `validate_unsigned`
            // while you can always require signed txs
            InvalidTransaction::Call.into()
        }
    }

    /// Parachain ID of the Kylin Oracle chain
    #[pallet::storage]
    #[pallet::getter(fn get_kylin_id)]
    pub(super) type KylinParaId<T: Config> = StorageValue<_, ParaId, OptionQuery>;

    /// Storage map for the feed URL Endpoint
    #[pallet::storage]
	#[pallet::getter(fn api_feeds)]
	pub type ApiFeeds<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, Vec<u8>, ApiFeed<T::BlockNumber>>;

}

/// Feed URL Endpoint data structure
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ApiFeed<BlockNumber> {
    requested_block_number: BlockNumber,
    url: Option<Vec<u8>>,
}

/// :TODO: Crypto price data structure, hardcoded for now, 
/// need to use more flexible type struct later 
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

        // :TODO: now we handle URL Endpoint result with specific hardcoded keys,
        // need to handle dynamic result type later 
        
        // SBP-M1 review: use some const values not to repeat yourself
        // And be more self-explaining
        // Also I can see code duplication between CCApi & CWApi
        // Refactor needed
        let mut values = Vec::<(Vec<u8>, i64)>::new();
        for (acc, key, val) in <ApiFeeds<T> as IterableStorageDoubleMap<_, _, _>>::iter() {
            // let mut response :Vec<u8>;
            if val.url.is_some() {
                // SBP-M1 review: use proper error handling instead of unwrap
                let response = Self::fetch_http_get_result(val.url.clone().unwrap())
                    .map_err(|_| "Failed fetch http")?;

                match str::from_utf8(&key) {
                    Ok("CCApi") => {
                        let price: CryptoComparePrice = serde_json::from_slice(&response)
                            .map_err(|_| "Response JSON was not well-formatted")?;
                        // We only store int, so every float will be convert to int with 6 decimals pad
                        let pval :i64 = (price.usdt * 1000000.0) as i64;
                        values.push((key.clone(), pval));
                    },
                    Ok("CWApi") => {
                        let price: CryptoComparePrice = serde_json::from_slice(&response)
                            .map_err(|_| "Response JSON was not well-formatted")?;
                        // We only store int, so every float will be convert to int with 6 decimals pad
                        let pval :i64 = (price.usdt * 1000000.0) as i64;
                        values.push((key.clone(), pval));
                    },
                    Ok(k) => {
                        log::debug!("No match API key [{:?}]", k);
                    },
                    _ => {},
                }
                
            };
        }

        if values.len() > 0 {
            if let Some(para_id) = <KylinParaId<T>>::get() {
                // write data to chain
                let results = signer.send_signed_transaction(|_account| Call::feed_data {
                    para_id: para_id,
                    values: values.clone(),
                });
                for (acc, res) in &results {
                    match res {
                        Ok(()) => log::info!("[{:?}] Submitted data", acc.id),
                        Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
                    }
                }
            }
        }

        Ok(())
    }

    /// Fetch current price and return the result in cents.
    // SBP-M1 review: introduce constants instead of hardcoded values
    // Apply proper error handling instead of unwrap
    // Check response length for security reasons
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

    // SBP-M1 review: introduce constant values instead of hardcoded ones
    fn feed_data_to_parachain(para_id: ParaId, values: Vec<(Vec<u8>, i64)>) -> DispatchResult {
        let remark = KylinMockCall::KylinOraclePallet(KylinMockFunc::xcm_feed_data {
            values,
        });
        match T::XcmSender::send_xcm(
            (
                1,
                Junction::Parachain(para_id.into()),
            ),
            Xcm(vec![Transact {
                origin_type: OriginKind::Native,
                require_weight_at_most: 1_000_000_000,
                call: remark.encode().into(),
            }]),
        ) {
            Ok(()) => {
                Self::deposit_event(Event::FeedDataSent(para_id))
            },
            Err(e) => {
                Self::deposit_event(Event::FeedDataError(e, para_id))
            },
        }

        Ok(())
    }

}
