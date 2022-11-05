use codec::{Decode, Encode, MaxEncodedLen};
use kylin_support::types::{
	EcdsaSignature, EthereumAddress,
};
use scale_info::TypeInfo;
use sp_runtime::{MultiSignature, RuntimeDebug};

/// A single Distribution.
#[derive(Encode, Decode, PartialEq, Eq, Copy, Clone, TypeInfo, MaxEncodedLen)]
pub struct Distribution<AccountId, Balance, Moment> {
	/// Creator of the Distribution.
	pub creator: AccountId,
	/// Total funds committed to the Distribution.
	pub total_funds: Balance,
	/// Total number of recipients
	pub total_recipients: u32,
	/// Amount of the `total_funds` already claimed.
	pub claimed_funds: Balance,
	/// Starting block of the Distribution.
	pub start: Option<Moment>,
	/// The minimum time, in blocks, between recipient claims.
	pub schedule: Moment,
	/// Set `true` if an distribution has been explicitly disabled.
	pub disabled: bool,
}

/// Funds, and related information, to be claimed by an Distribution recipient.
#[derive(Encode, Decode, PartialEq, Eq, Copy, Clone, TypeInfo, MaxEncodedLen)]
pub struct RecipientFund<Balance, Period> {
	/// Total funds committed for this recipient.
	pub total: Balance,
	/// Amount of the `total` this recipient has claimed.
	pub claimed: Balance,
	/// The minimum time, in blocks, between recipient claims.
	pub vesting_period: Period,
	/// If claims by this user will be funded by an external pool.
	pub funded_claim: bool,
}

/// Current State of an [`Distribution`](Distribution).
#[derive(Debug, Encode, Decode, PartialEq, Eq, Copy, Clone, TypeInfo, MaxEncodedLen)]
pub enum DistributionState {
	/// The Distribution has been created but has not started.
	Created,
	/// The Distribution has started. Recipients can claim funds.
	Enabled,
	/// The Distribution has ended. Recipients can **NOT** claim funds.
	Disabled,
} 