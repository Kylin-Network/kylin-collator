//! A set of constant values used in altair runtime

/// Money matters.
pub mod currency {
	use runtime_common::*;

	pub const MICRO_PCHU: Balance = MICRO_KYL;
	pub const MILLI_PCHU: Balance = MILLI_KYL;
	pub const CENTI_PCHU: Balance = CENTI_KYL;
	pub const PCHU: Balance = KYL;
}
