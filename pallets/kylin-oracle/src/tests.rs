// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate as kylin_oracle;
use crate::*;
use codec::Decode;
use frame_support::{
    parameter_types,
    traits::Everything,
    weights::{IdentityFee, Weight},
};

use sp_core::{
    offchain::{testing, OffchainWorkerExt, TransactionPoolExt},
    sr25519::Signature,
    H256,
};
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};

use sp_std::str;
use sp_std::vec::Vec;
use std::sync::Arc;

use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
};
use xcm_builder::{
    AllowUnpaidExecutionFrom, EnsureXcmOrigin, FixedWeightBounds, SignedToAccountId32,
    LocationInverter
};

use xcm_executor::{
    traits::{TransactAsset, WeightTrader},
    Assets, XcmExecutor,
};

use sp_core::{ sr25519, Pair, Public};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type AccountPublic = <Signature as Verify>::Signer;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        KylinOracle: kylin_oracle::{Pallet, Call, Storage, Event<T>, ValidateUnsigned},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
        XcmPallet: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin} = 51,
        CumulusXcm: cumulus_pallet_xcm::{Pallet, Call, Event<T>, Origin} = 52,

    }
);

parameter_types! {
    pub const TransactionByteFee: u64 = 1;
    pub const OperationalFeeMultiplier: u8 = 5;
}
impl pallet_transaction_payment::Config for Test {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type TransactionByteFee = TransactionByteFee;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
    type WeightToFee = IdentityFee<u64>;
    type FeeMultiplierUpdate = ();
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
    pub Ancestry: MultiLocation = Here.into();
}
pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;
pub type Barrier = AllowUnpaidExecutionFrom<Everything>;

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = sp_core::sr25519::Public;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

type Extrinsic = TestXt<Call, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

impl frame_system::offchain::SigningTypes for Test {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    type OverarchingCall = Call;
    type Extrinsic = Extrinsic;
}

parameter_types! {
    pub const GracePeriod: u64 = 5;
    pub const UnsignedInterval: u64 = 128;
    pub const UnsignedPriority: u64 = 1 << 20;
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 5;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
    type Balance = u64;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}
pub struct DoNothingRouter;
impl SendXcm for DoNothingRouter {
    fn send_xcm(_dest: impl Into<MultiLocation>, _msg: Xcm<()>) -> SendResult {
        Ok(())
    }
}
// For testing the module, we construct a mock runtime.

parameter_types! {
    pub const MinimumPeriod: u64 = 1;
}
impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

// pub trait Config: CreateSignedTransaction<Self> + frame_system::Config {}
impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        _public: <Signature as Verify>::Signer,
        _account: AccountId,
        nonce: u64,
    ) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
        Some((call, (nonce, ())))
    }
}

parameter_types! {
    // pub const RelayLocationson::X1(Parent);
    pub const RelayNetwork: NetworkId = NetworkId::Kusama;
    // pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
}
impl kylin_oracle::Config for Test {
    type Event = Event;
    type AuthorityId = crypto::TestAuthId;
    type Call = Call;
    type Origin = Origin;
    type XcmSender = DoNothingRouter;
    type UnsignedPriority = UnsignedPriority;
    type UnixTime = pallet_timestamp::Pallet<Test>;
    type Currency = Balances;
    type WeightInfo = ();
    type EstimateCallFee = TransactionPayment;
}

parameter_types! {
    pub const UnitWeightCost: Weight = 10;
    pub const MaxInstructions: u32 = 100;
}

pub struct DummyWeightTrader;
impl WeightTrader for DummyWeightTrader {
    fn new() -> Self {
        DummyWeightTrader
    }

    fn buy_weight(&mut self, _weight: Weight, _payment: Assets) -> Result<Assets, XcmError> {
        Ok(Assets::default())
    }
}
pub struct DummyAssetTransactor;
impl TransactAsset for DummyAssetTransactor {
    fn deposit_asset(_what: &MultiAsset, _who: &MultiLocation) -> XcmResult {
        Ok(())
    }

    fn withdraw_asset(_what: &MultiAsset, _who: &MultiLocation) -> Result<Assets, XcmError> {
        let asset: MultiAsset = (Parent, 100_000).into();
        Ok(asset.into())
    }
}
pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
    type Call = Call;
    type XcmSender = DoNothingRouter;
    type AssetTransactor = DummyAssetTransactor;
    type OriginConverter = pallet_xcm::XcmPassthrough<Origin>;
    type IsReserve = ();
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type Trader = DummyWeightTrader;
    type ResponseHandler = ();
    type AssetTrap = XcmPallet;
    type AssetClaims = XcmPallet;
    type SubscriptionService = XcmPallet;
}

parameter_types! {
    pub static AdvertisedXcmVersion: xcm::prelude::XcmVersion = 2;
}

impl pallet_xcm::Config for Test {
    type Event = Event;
    type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmRouter = DoNothingRouter;
    type LocationInverter = LocationInverter<Ancestry>;
    type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Everything;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type XcmReserveTransferFilter = Everything;
    type Origin = Origin;
    type Call = Call;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type AdvertisedXcmVersion = AdvertisedXcmVersion;
}

impl cumulus_pallet_xcm::Config for Test {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

#[test]
fn should_save_data_onchain_for_signed_data_submissions() {
    const PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();

    let keystore = KeyStore::new();

    SyncCryptoStore::sr25519_generate_new(
        &keystore,
        kylin_oracle::KEY_TYPE,
        Some(&format!("{}/hunter1", PHRASE)),
    )
    .unwrap();
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");

    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt(Arc::new(keystore)));
    mock_submit_response(&mut offchain_state.write());
    let expected_response = br#"{"USD": 155.23}"#.to_vec();
    t.execute_with(|| {
        KylinOracle::submit_price_feed(
            Origin::signed(alice),
            None,
            str::from_utf8(b"btc_usd").unwrap().as_bytes().to_vec(),
        )
        .unwrap();
        KylinOracle::fetch_data_and_send_signed(1).unwrap();

        let tx1 = pool_state.write().transactions.pop().unwrap();
        let tx2 = pool_state.write().transactions.pop().unwrap();

        let tx1 = Extrinsic::decode(&mut &*tx1).unwrap();
        let tx2 = Extrinsic::decode(&mut &*tx2).unwrap();

        if let Call::KylinOracle(crate::Call::submit_data_signed {
            block_number: _block_number,
            key: _key,
            data: response,
        }) = tx2.call
        {
            assert_eq!(response, expected_response);
        }

        if let Call::KylinOracle(crate::Call::clear_processed_requests_unsigned {
            block_number: _block_number,
            processed_requests,
        }) = tx1.call
        {
            assert_eq!(1, processed_requests.len());
        }
    });
}

#[test]
fn should_save_data_onchain_for_unsigned_submissions() {
    // const PHRASE: &str =
    // "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();

    let keystore = KeyStore::new();
    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt(Arc::new(keystore)));
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");

    mock_submit_response(&mut offchain_state.write());
    let expected_response = br#"{"USD": 155.23}"#.to_vec();
    t.execute_with(|| {
        KylinOracle::submit_price_feed(
            Origin::signed(alice),
            None,
            str::from_utf8(b"btc_usd").unwrap().as_bytes().to_vec(),
        )
        .unwrap();
        KylinOracle::fetch_data_and_send_raw_unsigned(1).unwrap();

        let tx1 = pool_state.write().transactions.pop().unwrap();
        let tx2 = pool_state.write().transactions.pop().unwrap();

        let tx1 = Extrinsic::decode(&mut &*tx1).unwrap();
        let tx2 = Extrinsic::decode(&mut &*tx2).unwrap();

        if let Call::KylinOracle(crate::Call::submit_data_unsigned {
            block_number: _block_number,
            key: _key,
            data: response,
        }) = tx2.call
        {
            assert_eq!(response, expected_response);
        }

        if let Call::KylinOracle(crate::Call::clear_processed_requests_unsigned {
            block_number: _block_number,
            processed_requests,
        }) = tx1.call
        {
            assert_eq!(1, processed_requests.len());
        }
    });
}

#[test]
fn should_write_data_onchain_directly_for_signed_requests() {
    const PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    let (offchain, _offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();
    let keystore = KeyStore::new();
    SyncCryptoStore::sr25519_generate_new(
        &keystore,
        kylin_oracle::KEY_TYPE,
        Some(&format!("{}/hunter1", PHRASE)),
    )
    .unwrap();

    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt(Arc::new(keystore)));
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");

    let feed_name = b"test_feed".to_vec();
    let sample_data = b"{sample_data}".to_vec();
    t.execute_with(|| {
        KylinOracle::write_data_onchain(
            Origin::signed(alice),
            feed_name,
            sample_data.clone(),
        )
        .unwrap();
        KylinOracle::fetch_data_and_send_signed(1).unwrap();

        let tx1 = pool_state.write().transactions.pop().unwrap();
        let tx2 = pool_state.write().transactions.pop().unwrap();

        let tx1 = Extrinsic::decode(&mut &*tx1).unwrap();
        let tx2 = Extrinsic::decode(&mut &*tx2).unwrap();

        if let Call::KylinOracle(crate::Call::submit_data_signed {
            block_number: _block_number,
            key: _key,
            data: response,
        }) = tx2.call
        {
            assert_eq!(response, sample_data);
        }

        if let Call::KylinOracle(crate::Call::clear_processed_requests_unsigned {
            block_number: _block_number,
            processed_requests,
        }) = tx1.call
        {
            assert_eq!(1, processed_requests.len());
        }
    });
}

fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}


fn mock_submit_response(state: &mut testing::OffchainState) {
    state.expect_request(testing::PendingRequest {
        method: "GET".into(),
        uri: "https://api.kylin-node.co.uk/prices?currency_pairs=btc_usd".into(),
        response: Some(br#"{"USD": 155.23}"#.to_vec()),
        sent: true,
        ..Default::default()
    });

}

fn mock_post_response(state: &mut testing::OffchainState) {
    

    // state.expect_request(testing::PendingRequest {
    //     method: "GET".into(),
    //     uri: "https://api.kylin-node.co.uk/prices?currency_pairs=btc_usd".into(),
    //     response: Some(br#"{"USD": 155.23}"#.to_vec()),
    //     sent: true,
    //     ..Default::default()
    // });


    let sample_body:Vec<u8> =  vec![123, 10, 32, 32, 32, 32, 34, 100, 97, 116, 97, 34, 58, 32, 123, 10, 32, 32, 32, 32, 32, 32, 32, 32, 34, 112, 97, 114, 97, 95, 105, 100, 34, 58, 32, 110, 117, 108, 108, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 34, 97, 99, 99, 111, 117, 110, 116, 95, 105, 100, 34, 58, 32, 34, 100, 52, 51, 53, 57, 51, 99, 55, 49, 53, 102, 100, 100, 51, 49, 99, 54, 49, 49, 52, 49, 97, 98, 100, 48, 52, 97, 57, 57, 102, 100, 54, 56, 50, 50, 99, 56, 53, 53, 56, 56, 53, 52, 99, 99, 100, 101, 51, 57, 97, 53, 54, 56, 52, 101, 55, 97, 53, 54, 100, 97, 50, 55, 100, 34, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 34, 114, 101, 113, 117, 101, 115, 116, 101, 100, 95, 98, 108, 111, 99, 107, 95, 110, 117, 109, 98, 101, 114, 34, 58, 32, 48, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 34, 112, 114, 111, 99, 101, 115, 115, 101, 100, 95, 98, 108, 111, 99, 107, 95, 110, 117, 109, 98, 101, 114, 34, 58, 32, 48, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 34, 114, 101, 113, 117, 101, 115, 116, 101, 100, 95, 116, 105, 109, 101, 115, 116, 97, 109, 112, 34, 58, 32, 48, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 34, 112, 114, 111, 99, 101, 115, 115, 101, 100, 95, 116, 105, 109, 101, 115, 116, 97, 109, 112, 34, 58, 32, 48, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 34, 112, 97, 121, 108, 111, 97, 100, 34, 58, 32, 34, 123, 115, 97, 109, 112, 108, 101, 95, 100, 97, 116, 97, 125, 34, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 34, 102, 101, 101, 100, 95, 110, 97, 109, 101, 34, 58, 32, 34, 112, 114, 105, 99, 101, 95, 102, 101, 101, 100, 105, 110, 103, 34, 44, 10, 32, 32, 32, 32, 32, 32, 32, 32, 34, 117, 114, 108, 34, 58, 32, 34, 104, 116, 116, 112, 115, 58, 47, 47, 97, 112, 105, 46, 107, 121, 108, 105, 110, 45, 110, 111, 100, 101, 46, 99, 111, 46, 117, 107, 47, 112, 114, 105, 99, 101, 115, 63, 99, 117, 114, 114, 101, 110, 99, 121, 95, 112, 97, 105, 114, 115, 61, 98, 116, 99, 95, 117, 115, 100, 34, 10, 32, 32, 32, 32, 125, 44, 10, 32, 32, 32, 32, 34, 104, 97, 115, 104, 34, 58, 32, 34, 57, 57, 100, 48, 97, 51, 99, 97, 55, 98, 102, 53, 56, 57, 100, 99, 55, 57, 57, 51, 49, 97, 53, 101, 51, 49, 48, 98, 99, 101, 100, 51, 100, 57, 55, 55, 57, 48, 54, 98, 99, 50, 56, 57, 54, 101, 97, 49, 99, 101, 99, 97, 53, 97, 56, 50, 52, 101, 57, 49, 97, 57, 53, 49, 34, 10, 125];

    let mut pending_request = testing::PendingRequest {
        method: "POST".into(),
        uri: "https://api.kylin-node.co.uk/submit".into(),
        body: sample_body,
        response: Some(br#"{"USD": 155.23}"#.to_vec()),
        sent: true,
        ..Default::default()
    };

    pending_request.headers.push(("x-api-key".into(), "test_api_key".into()));
    pending_request.headers.push(("content-type".into(), "application/json".into()));
    state.expect_request(pending_request);
}

fn mock_query_response(state: &mut testing::OffchainState) {
    let pending_request = testing::PendingRequest {
        // 99d0a3ca7bf589dc79931a5e310bced3d977906bc2896ea1ceca5a824e91a951
        method: "GET".into(),
        uri: "https://api.kylin-node.co.uk/query?hash=99d0a3ca7bf589dc79931a5e310bced3d977906bc2896ea1ceca5a824e91a951".into(),
        response: Some(br#"{"USD": 155.23}"#.to_vec()),
        sent: true,
        ..Default::default()
    };
    state.expect_request(pending_request);
}

#[test]
fn should_award_query_fees() {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let additional_amount = 10_000;
    // let query_fee = 31252504;

    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, _pool_state) = testing::TestTransactionPoolExt::new();

    let keystore = KeyStore::new();
    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt(Arc::new(keystore)));
    t.execute_with(|| {
        let _ = <pallet_balances::Pallet<Test> as Currency<AccountId>>::deposit_creating(
                        &alice,
                        additional_amount
        );

        let _ =  <pallet_balances::Pallet<Test> as Currency<AccountId>>::deposit_creating(
                        &bob,
                        additional_amount
                    );
        
        let initial_alice_balance = <pallet_balances::Pallet<Test> as Currency<AccountId>>::free_balance(&alice);

        let sample_data = b"{sample_data}".to_vec();
        let mut processed_requests: Vec<u64> = Vec::new();
        let mut key = 10000000u64;
        let mut alice_fee = 0;
        let mut bob_fee = 0;
        processed_requests.push(key);

        // key = 10000000u64, data index = 10000000u64 weight = 65_955_000 + 100 * 3 + 1000 * 3
    //     (65_955_000 as Weight)
    //         .saturating_add(RocksDbWeight::get().reads(3 as Weight))
    //         .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    // }
        let submit_price_feed_fee = 65_955_000 + 100 * 3 + 1000 * 3;
        alice_fee += submit_price_feed_fee;
        println!("alice: submit_price_feed fee {}", submit_price_feed_fee);
        mock_submit_response(&mut offchain_state.write());       
        KylinOracle::submit_price_feed(
            Origin::signed(alice),
            None,
            str::from_utf8(b"btc_usd").unwrap().as_bytes().to_vec(),
        )
        .unwrap();
        // (65_955_000 as Weight)
        // .saturating_add(T::DbWeight::get().reads(3 as Weight))
        // .saturating_add(T::DbWeight::get().writes(3 as Weight))
        let submit_data_signed_feed_fee = 65_955_000 + 100 * 3 + 1000 * 3;
        alice_fee += submit_data_signed_feed_fee;
        println!("alice: submit_price_feed fee {}", submit_data_signed_feed_fee);
        KylinOracle::submit_data_signed(
            Origin::signed(alice),
            1,
            key,
            sample_data.clone()
        )
        .unwrap();
        KylinOracle::fetch_data_and_send_raw_unsigned(1).unwrap();
        KylinOracle::clear_processed_requests_unsigned(
            Origin::none(),
            1,
            processed_requests.clone(),
        )
        .unwrap();
        mock_post_response(&mut offchain_state.write());
        KylinOracle::fetch_data_and_send_raw_unsigned(2).unwrap();
    
        // key = 10000001u64, data index = 10000001u64
        key = 10000001u64;
        mock_query_response(&mut offchain_state.write());
        // (121_180_000 as Weight) // Standard Error: 0
        // .saturating_add(RocksDbWeight::get().reads(4 as Weight))
        // .saturating_add(RocksDbWeight::get().writes(2 as Weight))
        let query_data_fee = 121_180_000 + 100 * 4 + 1000 * 3;
        bob_fee += query_data_fee;
        println!("bob: query_data fee {}", query_data_fee);
        KylinOracle::query_data(
            Origin::signed(bob),
            None,
            str::from_utf8(b"price_feeding").unwrap().as_bytes().to_vec(),
        )
        .unwrap();
        // (20_716_000 as Weight) // Standard Error: 0
        // .saturating_add(RocksDbWeight::get().reads(2 as Weight))
        // .saturating_add(RocksDbWeight::get().writes(1 as Weight))
        let submit_data_signed_fee = 20_716_000 + 100 * 2 + 1000 * 1;
        bob_fee += submit_data_signed_fee;
        println!("bob: submit_data_signed fee {}", submit_data_signed_fee);
        KylinOracle::submit_data_signed(
            Origin::signed(bob),
            2,
            key,
            sample_data.clone()
        )
        .unwrap();
        KylinOracle::clear_api_queue_unsigned(
            Origin::none(),
            2,
            processed_requests.clone(),
        )
        .unwrap();
        KylinOracle::fetch_data_and_send_raw_unsigned(3).unwrap();


        let free_alice_balance = <pallet_balances::Pallet<Test> as Currency<AccountId>>::free_balance(&alice);
        println!("init alice balance {}, free alice balance {}, alice fee used {}, bob fee used {}", initial_alice_balance, free_alice_balance, alice_fee, bob_fee);
        // TODO adjust fee
        // assert_eq!(
        //     free_alice_balance + query_fee,
        //     initial_alice_balance,
        // );
    });

}