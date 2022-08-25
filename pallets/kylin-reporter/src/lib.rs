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
            
            //let res = Self::fetch_api_and_send_signed(block_number);

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

        #[pallet::weight(0)]
        pub fn xcm_receive_data(
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


        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn xcm_submit_data(
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
        /// New feed is submitted.
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

    #[pallet::storage]
	#[pallet::getter(fn api_feeds)]
	pub type ApiFeeds<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::OracleKey, ApiFeed<ParaId, T::BlockNumber>>;

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
