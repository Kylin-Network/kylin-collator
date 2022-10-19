// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
use codec::{Decode, Encode};

use sp_api::impl_runtime_apis;
use sp_core::OpaqueMetadata;
use sp_runtime::{
	create_runtime_str, generic,
	generic::Era,
	impl_opaque_keys,
	traits::{
		AccountIdLookup, BlakeTwo256, Block as BlockT, Convert, ConvertInto,
		Extrinsic as ExtrinsicT, Verify, AccountIdConversion
	},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, Percent, Permill, Perbill, RuntimeDebug
};

use sp_std::{marker::PhantomData, convert::TryInto, convert::TryFrom, prelude::*};

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
// use crate::sp_api_hidden_includes_IMPL_RUNTIME_APIS::sp_api::Encode;
pub use frame_support::{
	construct_runtime, ensure, match_types, parameter_types,
	traits::{
		Contains, EitherOfDiverse, EqualPrivilegeOnly, Everything, 
		IsInVec, Randomness, Nothing, ConstU32, ConstU64, ConstU128
	},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass, IdentityFee, Weight, ConstantMultiplier
	},
	PalletId, StorageValue,
};
use frame_support::traits::AsEnsureOriginWithArg;
use frame_system::EnsureSigned;
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot, 
};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

pub use cumulus_primitives_core::ParaId;

use xcm_executor::{
	traits::{ShouldExecute},
	XcmExecutor,
};

// XCM imports
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, EnsureXcmOrigin, FixedWeightBounds, FixedRateOfFungible,
	LocationInverter, ParentAsSuperuser, ParentIsPreset, RelayChainAsNative,
	SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit, AllowUnpaidExecutionFrom
};

//use kylin_oracle::DefaultCombineData;

/// common types for the runtime.
pub use runtime_common::*;

pub type ReserveIdentifier = [u8; 8];

pub type SessionHandlers = ();

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("kylin"),
	impl_name: create_runtime_str!("kylin"),
	authoring_version: 1,
	spec_version: 22,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
/// We allow for .5 seconds of compute with a 12 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = runtime_common::Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	type Hash = Hash;

	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type SS58Prefix = SS58Prefix;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 1 * MICRO_KYL;
	pub const TransferFee: Balance = 1 * MICRO_KYL;
	pub const CreationFee: Balance = 1 * MICRO_KYL;
	pub const TransactionByteFee: Balance = 1 * (MICRO_KYL / 100); // 10_000_000_000
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ReserveIdentifier;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const VotingPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * 24 * 60 * MINUTES;
	pub const MinimumDeposit: Balance = 100 * PCHU;
	pub const EnactmentPeriod: BlockNumber = 30 * 24 * 60 * MINUTES;
	pub const CooloffPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const MaxProposals: u32 = 100;
}

impl kylin_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type VoteLockingPeriod = EnactmentPeriod; // Same as EnactmentPeriod
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
	/// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>;
	type InstantOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>;
	type InstantAllowed = frame_support::traits::ConstBool<true>;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = frame_support::traits::ConstU32<100>;
	type WeightInfo = kylin_democracy::weights::SubstrateWeight<Runtime>;
	type MaxProposals = MaxProposals;
}


parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 5 * DAYS;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

type ApproveOrigin = EnsureRoot<AccountId>;
type EnsureRootOrHalfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
>;

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub ProposalBondMinimum: Balance = 5 * KYL;
	pub ProposalBondMaximum: Balance = 25 * KYL;
	pub const SpendPeriod: BlockNumber = 7 * DAYS;
	pub const Burn: Permill = Permill::from_percent(0);

	pub const TipCountdown: BlockNumber = DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(5);
	pub TipReportDepositBase: Balance = deposit(1, 0);
	pub BountyDepositBase: Balance = deposit(1, 0);
	pub const BountyDepositPayoutDelay: BlockNumber = 4 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 35 * DAYS;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub CuratorDepositMin: Balance = KYL;
	pub CuratorDepositMax: Balance = 100 * KYL;
	pub BountyValueMinimum: Balance = 5 * KYL;
	pub DataDepositPerByte: Balance = deposit(0, 1);
	pub const MaximumReasonLength: u32 = 8192;

	pub const SevenDays: BlockNumber = 7 * DAYS;
	pub const OneDay: BlockNumber = DAYS;
}

impl pallet_treasury::Config for Runtime {
    
	type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type ApproveOrigin = ApproveOrigin;
    type RejectOrigin = EnsureRootOrHalfCouncil;
    type Event = Event;
    type OnSlash = Treasury;
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BurnDestination = (); // TODO: Enable Society
    type SpendFunds = ();
	type ProposalBondMaximum = ProposalBondMaximum;
	type WeightInfo = ();
	type MaxApprovals = frame_support::traits::ConstU32<30>;
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<u128>;
}





impl pallet_randomness_collective_flip::Config for Runtime {}

parameter_types! {
	pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
	type Event = Event;
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = ();
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
}

impl pallet_sudo::Config for Runtime {
	type Call = Call;
	type Event = Event;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type OnSystemEvent = ();
	type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
	pub const DotLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = MultiCurrencyAdapter<
	OrmlCurrencies,
	OrmlUnknownTokens,
	IsNativeConcrete<CurrencyId, CurrencyIdConvert>,
	AccountId,
	LocationToAccountId,
	CurrencyId,
	CurrencyIdConvert,
	DepositToAlternative<NativeTreasuryAccount, OrmlCurrencies, CurrencyId, AccountId, Balance>,
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, Origin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, Origin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<Origin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `Origin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, Origin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<Origin>,
);

parameter_types! {
	// One XCM operation is 200_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = 200_000_000;
	// One ROC buys 1 second of weight.
	pub const WeightPrice: (MultiLocation, u128) = (MultiLocation::parent(), KYL);
	pub const MaxInstructions: u32 = 100;
}

match_types! {
	pub type ParentOrParentsExecutivePlurality: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(Plurality { id: BodyId::Unit, .. }) }
	};
}

match_types! {
	pub type Statemint: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: X1(Parachain(1000)) }
	};
}

match_types! {
	pub type SpecParachain: impl Contains<MultiLocation> = {
		MultiLocation {parents: 1, interior: X1(Parachain(1000))} |
		MultiLocation {parents: 1, interior: X1(Parachain(2000))}
	};
}

// pub type Barrier = (
// 	TakeWeightCredit,
// 	AllowAnyPaidExecutionFrom<Everything>,
// 	AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
//     // ^^^ Parent and its exec plurality get free execution
//     AllowUnpaidExecutionFrom<SpecParachain>,
// );
pub type Barrier = AllowUnpaidExecutionFrom<Everything>;

pub struct AllowAnyPaidExecutionFrom<T>(PhantomData<T>);
impl<T: Contains<MultiLocation>> ShouldExecute for AllowAnyPaidExecutionFrom<T> {
	fn should_execute<Call>(
		origin: &MultiLocation,
		_message: &mut Xcm<Call>,
		_max_weight: Weight,
		_weight_credit: &mut Weight,
	) -> Result<(), ()> {
		ensure!(T::contains(origin), ());
		Ok(())
	}
}
parameter_types! {
	pub StatemintLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(1000)));
}

parameter_types! {
	pub DotPerSecond: (AssetId, u128) = (MultiLocation::parent().into(), native_token_per_second() / 1_000_000);
	pub AcaPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(2000), GeneralKey([0, 128].to_vec().try_into().unwrap())),
		).into(),
		// ACA:KYL = 1:1_000_000  // ~80_000_000_000 amount
		native_token_per_second() / 1_000_000
	);
	pub AusdPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(2000), GeneralKey([0, 129].to_vec().try_into().unwrap())),
		).into(),
		// AUSD:KYL = 1:1_000_000
		native_token_per_second() / 1_000_000
	);
	pub LdotPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(2000), GeneralKey([0, 131].to_vec().try_into().unwrap())),
		).into(),
		// LDOT:KYL = 1:1_000_000
		native_token_per_second() / 1_000_000
	);
	pub NativeTokenPerSecond: (AssetId, u128) = (
		MultiLocation::new(0, X1(GeneralKey(b"KYL".to_vec().try_into().unwrap()))).into(),
		native_token_per_second()
	);
}

pub type Trader = (
    FixedRateOfFungible<DotPerSecond, ()>,
    FixedRateOfFungible<NativeTokenPerSecond, ()>,
    FixedRateOfFungible<AcaPerSecond, ()>,
    FixedRateOfFungible<AusdPerSecond, ()>,
    FixedRateOfFungible<LdotPerSecond, ()>,
);

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type Call = Call;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;
	// type IsTeleporter = NativeAsset; // <- should be enough to allow teleportation of ROC
	type IsTeleporter = ();
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type Trader = Trader;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
}

parameter_types! {
	pub const MaxDownwardMessageWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 10;
	pub const GracePeriod: u32 = 5;
	pub const UnsignedInterval: u64 = 128;
	pub const UnsignedPriority: u64 = 1 << 20;
}

pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Nothing;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type LocationInverter = LocationInverter<Ancestry>;
	type Origin = Origin;
	type Call = Call;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = frame_system::EnsureRoot<AccountId>;
}

impl kylin_reporter::Config for Runtime {
	type Event = Event;
	type AuthorityId = kylin_reporter::crypto::TestAuthId;
	type Call = Call;
	type Origin = Origin;
	type XcmSender = XcmRouter;
	type WeightInfo = kylin_reporter::weights::SubstrateWeight<Runtime>;
	type Currency = Balances;
	type Members = OracleProvider;
}

parameter_types! {
	pub const MinVestedTransfer: Balance = MIN_VESTING * KYL;
}

impl pallet_vesting::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting::weights::SubstrateWeight<Self>;
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

parameter_types! {
	pub const MinimumReward: Balance = 0;
	pub const Initialized: bool = false;
	pub const InitializationPayment: Perbill = Perbill::from_percent(30);
	pub const MaxInitContributorsBatchSizes: u32 = 500;
	pub const RelaySignaturesThreshold: Perbill = Perbill::from_percent(100);
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
	type PalletsOrigin = OriginCaller;
}

parameter_types! {
	pub const OracleProviderMaxMembers: u32 = 100;
}

impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrHalfCouncil;
	type RemoveOrigin = EnsureRootOrHalfCouncil;
	type SwapOrigin = EnsureRootOrHalfCouncil;
	type ResetOrigin = EnsureRootOrHalfCouncil;
	type PrimeOrigin = EnsureRootOrHalfCouncil;
	type MembershipInitialized = ();
	type MembershipChanged = Council;
	type MaxMembers = OracleProviderMaxMembers;
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = NORMAL_DISPATCH_RATIO * RuntimeBlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
	pub const NoPreimagePostponement: Option<u32> = Some(10);
}

parameter_types! {
	pub const PreimageMaxSize: u32 = 4096 * 1024;
	pub const PreimageBaseDeposit: Balance = runtime_common::deposit(2, 64);
	pub const PreimageByteDeposit: Balance = runtime_common::deposit(0, 1);
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = ();
	type Event = Event;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type MaxSize = PreimageMaxSize;
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Self>;
	type PreimageProvider = Preimage;
	type NoPreimagePostponement = NoPreimagePostponement;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		public: <Signature as Verify>::Signer,
		account: AccountId,
		nonce: <Runtime as frame_system::Config>::Index,
	) -> Option<(Call, <UncheckedExtrinsic as ExtrinsicT>::SignaturePayload)> {
		use sp_runtime::traits::StaticLookup;
		// take the biggest period possible.
		let period = BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;

		let current_block = System::block_number()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1)
			.into();
		let tip = 0;
		let era = Era::mortal(period, current_block);
		let extra = (
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(era),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;

		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let (call, extra, _) = raw_payload.deconstruct();
		let address = <Runtime as frame_system::Config>::Lookup::unlookup(account);
		Some((call, (address, signature, extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	Call: From<C>,
{
	type OverarchingCall = Call;
	type Extrinsic = UncheckedExtrinsic;
}

parameter_types! {
	pub const AssetDeposit: Balance = 1 * KYL;
	pub const AssetAccountDeposit: Balance = KYL;
	pub const ApprovalDeposit: Balance = 100 * MILLI_KYL;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 1 * KYL;
	pub const MetadataDepositPerByte: Balance = 10 * MILLI_KYL;
	pub const UnitBody: BodyId = BodyId::Unit;
	pub const MaxAuthorities: u32 = 100_000;
}

pub type AssetsForceOrigin = EnsureRoot<AccountId>;

impl pallet_assets::Config for Runtime {
	type Event = Event;
	type Balance = u64;
	type AssetId = parachains_common::AssetId;
	type Currency = Balances;
	type ForceOrigin = AssetsForceOrigin;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	type AssetAccountDeposit = AssetAccountDeposit;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = (CollatorSelection,);
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxCandidates: u32 = 1000;
	pub const MinCandidates: u32 = 5;
	pub const SessionLength: BlockNumber = 6 * HOURS;
	pub const MaxInvulnerables: u32 = 100;
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

// We allow root only to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureRoot<AccountId>;

impl pallet_collator_selection::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type PotId = PotId;
	type MaxCandidates = MaxCandidates;
	type MinCandidates = MinCandidates;
	type MaxInvulnerables = MaxInvulnerables;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type Event = Event;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = ();
}

// orml pallets
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use orml_traits::{location::AbsoluteReserveProvider, parameter_type_with_key};
use orml_xcm_support::{
    DepositToAlternative, IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset,
};

impl orml_xcm::Config for Runtime {
	type Event = Event;
	type SovereignOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	pub SelfLocation: MultiLocation =  MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
	pub const BaseXcmWeight: Weight = 100_000_000;
	pub const MaxAssetsForTransfer: usize = 2;
	pub const TreasuryPalletId: PalletId = PalletId(*b"kylntrsy");
}

// kylin CurrencyId
#[derive(
	Encode,
	Decode,
	Eq,
	PartialEq,
	Copy,
	Clone,
	RuntimeDebug,
	PartialOrd,
	Ord,
	codec::MaxEncodedLen,
	scale_info::TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	DOT,
	KYL,
	ACA,
	LDOT,
	AUSD,
	MOVR, 
	BNC,
	KTON,
	RING
}


pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: AccountId) -> MultiLocation {
		X1(Junction::AccountId32 {
			network: NetworkId::Any,
			id: account.into(),
		})
		.into()
	}
}

pub struct CurrencyIdConvert;
// CurrencyId -> MultiLaction
impl Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdConvert {
	fn convert(currency: CurrencyId) -> Option<MultiLocation> {
		match currency {
			CurrencyId::KYL => Some(MultiLocation::new(
				1,
				X2(
					Parachain(ParachainInfo::parachain_id().into()),
					GeneralKey(b"KYL".to_vec().try_into().unwrap()),
				),
			)),
			// Kusama statemine paraid 1000
			CurrencyId::DOT => Some(MultiLocation::parent()),
			// acala paraid 2000
			CurrencyId::ACA => Some(MultiLocation::new(
				1,
				X2(Parachain(2000), GeneralKey([0, 0].to_vec().try_into().unwrap())),
			)),
			CurrencyId::AUSD => Some(MultiLocation::new(
				1,
				X2(Parachain(2000), GeneralKey([0, 1].to_vec().try_into().unwrap())),
			)),
			CurrencyId::LDOT => Some(MultiLocation::new(
				1,
				X2(Parachain(2000), GeneralKey([0, 3].to_vec().try_into().unwrap())),
			)),
			CurrencyId::MOVR => Some(MultiLocation::new(
				1,
				X2(Parachain(2024), GeneralKey([0, 132].to_vec().try_into().unwrap())),
			)),
			CurrencyId::BNC => Some(MultiLocation::new(
				1,
				X2(Parachain(2030), GeneralKey(b"BNC".to_vec().try_into().unwrap())),
			)),
			CurrencyId::RING => Some(MultiLocation::new(
				1,
				X2(Parachain(2046), GeneralKey(b"RING".to_vec().try_into().unwrap())),
			)),
			CurrencyId::KTON => Some(MultiLocation::new(
				1,
				X2(Parachain(2046), GeneralKey(b"KTON".to_vec().try_into().unwrap())),
			)),
		}
	}
}
// MultiLaction -> CurrencyId
impl Convert<MultiLocation, Option<CurrencyId>> for CurrencyIdConvert {
	fn convert(location: MultiLocation) -> Option<CurrencyId> {
		if location == MultiLocation::parent() {
			return Some(CurrencyId::DOT);
		}

		let aca: Vec<u8> = [0, 0].to_vec();
		let ausd: Vec<u8> = [0, 1].to_vec();
		let ldot: Vec<u8> = [0, 3].to_vec();
		let kylin: Vec<u8> = "KYL".into();

		match location {
			MultiLocation { parents, interior } if parents == 1 => match interior {
				X2(Parachain(2000), GeneralKey(k)) if k == aca => Some(CurrencyId::ACA),
				X2(Parachain(2000), GeneralKey(k)) if k == ausd => Some(CurrencyId::AUSD),
				X2(Parachain(2000), GeneralKey(k)) if k == ldot => Some(CurrencyId::LDOT),
				X2(Parachain(id), GeneralKey(k))
					if ParaId::from(id) == ParachainInfo::parachain_id() && k == kylin =>
				{
					Some(CurrencyId::KYL)
				}
				_ => None,
			},
			MultiLocation { parents, interior } if parents == 0 => match interior {
				X1(GeneralKey(k)) if k == kylin => Some(CurrencyId::KYL),
				X1(GeneralKey(k)) if k == aca => Some(CurrencyId::ACA),
				X1(GeneralKey(k)) if k == ausd => Some(CurrencyId::AUSD),
				X1(GeneralKey(k)) if k == ldot => Some(CurrencyId::LDOT),
				_ => None,
			},
			_ => None,
		}
	}
}
// Asset -> CurrencyId
impl Convert<MultiAsset, Option<CurrencyId>> for CurrencyIdConvert {
	fn convert(a: MultiAsset) -> Option<CurrencyId> {
		if let MultiAsset {
			fun: Fungible(_),
			id: Concrete(id),
		} = a
		{
			Self::convert(id)
		} else {
			Option::None
		}
	}
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = CurrencyId::KYL;
}
pub type Amount = i128;

impl orml_currencies::Config for Runtime {
    type MultiCurrency = OrmlTokens;
    type NativeCurrency = orml_currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}


parameter_type_with_key! {
	pub ParachainMinFee: |location: MultiLocation| -> Option<u128> {
		#[allow(clippy::match_ref_pats)] // false positive
		match (location.parents, location.first_interior()) {
			(1, Some(Parachain(2102))) => Some(40),
			_ => None,
		}
	};
}

// The XCM message wrapper wrapper
impl orml_xtokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type CurrencyIdConvert = CurrencyIdConvert;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type SelfLocation = SelfLocation;
	// Take note that this pallet does not have the typical configurable WeightInfo.
	// It uses the Weigher configuration to calculate weights for the user callable extrinsics on this chain,
	// as well as weights for execution on the destination chain. Both based on the composed xcm messages.
	type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type LocationInverter = LocationInverter<Ancestry>;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
	type MinXcmFee = ParachainMinFee;
	type ReserveProvider = AbsoluteReserveProvider;
	type MultiLocationsFilter = Everything;
}

parameter_type_with_key! {
    pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
        // every currency has a zero existential deposit
        match currency_id {
            _ => 0,
        }
    };
}

parameter_types! {
    pub ORMLMaxLocks: u32 = 2;
    pub NativeTreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
}

impl orml_unknown_tokens::Config for Runtime {
    type Event = Event;
}

impl orml_tokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type OnDust = orml_tokens::TransferDust<Runtime, NativeTreasuryAccount>;
    type MaxLocks = ORMLMaxLocks;
    type DustRemovalWhitelist = Nothing;
	type OnNewTokenAccount =  ();
	type OnKilledTokenAccount =  ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ReserveIdentifier;
}

parameter_types! {
	pub const CollectionDeposit: Balance = 10 * CENTI_PCHU;
	pub const ItemDeposit: Balance = PCHU;
	pub const KeyLimit: u32 = 32;
	pub const ValueLimit: u32 = 256;
	pub const UniquesMetadataDepositBase: Balance = 10 * CENTI_PCHU;
	pub const AttributeDepositBase: Balance = 10 * CENTI_PCHU;
	pub const DepositPerByte: Balance = CENTI_PCHU;
	pub const UniquesStringLimit: u32 = 512;
	pub const MaxPropertiesPerTheme: u32 = 100;
	pub const MaxCollectionsEquippablePerPart: u32 = 100;
}


impl pallet_uniques::Config for Runtime {
	type Event = Event;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type Locker = kylin_feed::Pallet<Runtime>;
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = UniquesMetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;
	type StringLimit = UniquesStringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxRecursions: u32 = 10;
	pub const ResourceSymbolLimit: u32 = 10;
	pub const PartsLimit: u32 = 25;
	pub const MaxPriorities: u32 = 25;
	pub const CollectionSymbolLimit: u32 = 100;
	pub const MaxResourcesOnMint: u32 = 100;
}

impl kylin_feed::Config for Runtime {
	type Event = Event;
	//type ProtocolOrigin = frame_system::EnsureRoot<AccountId>;
	type Origin = Origin;
	type MaxRecursions = MaxRecursions;
	type ResourceSymbolLimit = ResourceSymbolLimit;
	type PartsLimit = PartsLimit;
	type MaxPriorities = MaxPriorities;
	type CollectionSymbolLimit = CollectionSymbolLimit;
	type MaxResourcesOnMint = MaxResourcesOnMint;
	type XcmSender = XcmRouter;
}

impl kylin_feedback::Config for Runtime {
	type Event = Event;
	//type ProtocolOrigin = frame_system::EnsureRoot<AccountId>;
	type Origin = Origin;
}

construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = generic::Block<Header, sp_runtime::OpaqueExtrinsic>,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// System support stuff.
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		ParachainSystem: cumulus_pallet_parachain_system::{
			Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,
		} = 1,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 2,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 3,
		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>} = 4,
		Scheduler: pallet_scheduler::{Pallet, Storage, Event<T>, Call} = 5,
		Utility: pallet_utility::{Pallet, Call, Event} = 6,
		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 7,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 8,

		// Monetary stuff.
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 11,
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>} = 12,
		Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>} = 13,

		// Collator support. The order of these 4 are important and shall not change.
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 20,
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 22,
		Aura: pallet_aura::{Pallet, Storage, Config<T>} = 23,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 24,


		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 50,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin, Config} = 51,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Call, Event<T>, Origin} = 52,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 53,

		// Kylin Pallets
		OracleProvider: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>} = 54,
		KylinReporterPallet: kylin_reporter::{Pallet, Call, Storage, Event<T>, ValidateUnsigned} = 55,

		// orml
		OrmlXcm: orml_xcm::{Pallet, Call, Event<T>} = 70,
		OrmlXTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>} = 71,
		OrmlTokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>} = 72,
		OrmlUnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event} = 73,
		OrmlCurrencies: orml_currencies::{Pallet, Call} = 74,
		//Democracy
		Democracy: kylin_democracy = 90,
		Council: pallet_collective::<Instance1> = 91,
		TechnicalCommittee: pallet_collective::<Instance2> = 92,
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 93,
		Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>} = 94,
		KylinFeed: kylin_feed::{Pallet, Call, Event<T>, Storage} = 95,
		KylinFeedback: kylin_feedback::{Pallet, Call, Event<T>, Storage} = 96,
		
	}
}

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem, // by bart: https://github.com/paritytech/substrate/pull/10043
	(),
>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[frame_benchmarking, BaselineBench::<Runtime>]
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_timestamp, Timestamp]
		[proposals, ImbueProposals]
	);
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, runtime_common::Index> for Runtime {
		fn account_nonce(account: AccountId) -> runtime_common::Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}
	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
				config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString>{
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey, add_benchmark};

			// you can whitelist any storage keys you do not want to track here
			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, kylin_oracle, KylinOraclePallet);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}

		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmark!(list, extra, kylin_oracle, KylinOraclePallet);

			let storage_info = AllPalletsWithSystem::storage_info();

			return (list, storage_info)
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data =
			cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
				relay_chain_slot,
				sp_std::time::Duration::from_secs(6),
			)
			.create_inherent_data()
			.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(&block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
