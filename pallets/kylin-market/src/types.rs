use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ListInfo<AccountId, Balance, BlockNumber> {
	pub(super) listed_by: AccountId,
	pub(super) amount: Balance,
	pub(super) expires: Option<BlockNumber>,
}

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Offer<AccountId, Balance, BlockNumber> {
	pub(super) maker: AccountId,
	pub(super) amount: Balance,
	pub(super) expires: Option<BlockNumber>,
}
