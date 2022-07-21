/// Error type for interfacing with the novel poly basis
#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum Error {
	#[error("Number of wanted shards {0} exceeds max of 2^16")]
	WantedShardCountTooHigh(usize),

	#[error("Number of wanted shards must be at least 2, but is {0}")]
	WantedShardCountTooLow(usize),

	#[error("Number of wanted payload shards must be at least 1, but is {0}")]
	WantedPayloadShardCountTooLow(usize),

	#[error("Size of the payload is zero")]
	PayloadSizeIsZero,

	#[error("Needs at least {min} shards of {all} to recover, have {have}")]
	NeedMoreShards { have: usize, min: usize, all: usize },

	#[error("Parameters: n (= {n}) and k (= {k}) both must be a power of 2")]
	ParamterMustBePowerOf2 { n: usize, k: usize },

	#[error("Shards do have inconsistent lengths: first = {first}, other = {other})")]
	InconsistentShardLengths { first: usize, other: usize },
}

/// Result alias to simplify API.
pub type Result<T> = std::result::Result<T, Error>;
