//! A set of constant values used in altair runtime

/// Money matters.
pub mod currency {
	use runtime_common::*;

	pub const MICRO_PCHU: Balance = MICRO_KYL;
	pub const MILLI_PCHU: Balance = MILLI_KYL;
	pub const CENTI_PCHU: Balance = CENTI_KYL;
	pub const PCHU: Balance = KYL;
}
pub mod time {
	use primitives::{Balance, BlockNumber, Moment};

	pub const MILLISECS_PER_BLOCK: Moment = 6000;
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;
	pub const EPOCH_DURATION_IN_SLOTS: BlockNumber = prod_or_fast!(1 * HOURS, 1 * MINUTES);

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
	pub const WEEKS: BlockNumber = DAYS * 7;

	// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}
