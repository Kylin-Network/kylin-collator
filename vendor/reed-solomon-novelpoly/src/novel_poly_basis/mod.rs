// Encoding/erasure decoding for Reed-Solomon codes over binary extension fields
//
// Derived impl of `RSAErasureCode.c`.
//
// Lin, Han and Chung, "Novel Polynomial Basis and Its Application to Reed-Solomon Erasure Codes," FOCS14.
// (http://arxiv.org/abs/1404.3458)

use crate::errors::*;
use crate::f2e16::*;
use crate::Shard;

mod encode;
mod reconstruct;

pub use self::encode::*;
pub use self::reconstruct::*;
pub use super::util::*;

use super::field::f2e16;

/// Params for the encoder / decoder
/// derived from a target validator count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeParams {
	/// total number of message symbols to send
	/// Invariant is a power of base 2
	n: usize,
	/// number of information containing chunks
	/// Invariant is a power of base 2, `k < n`
	k: usize,
	/// Avoid copying unnecessary chunks.
	wanted_n: usize,
}

impl CodeParams {
	/// Create a new reed solomon erasure encoding wrapper
	/// `k` the intended number of data shards needed to recover.
	/// `n` the intended number of resulting shards.
	///
	/// Assures that the derived paramters retain at most the given coding
	/// rate, and as such assure recoverability with at least an equiv fraction
	/// as provided by the input `n`, and `k` parameterset.
	pub fn derive_parameters(n: usize, k: usize) -> Result<Self> {
		if n < 2 {
			return Err(Error::WantedShardCountTooLow(n));
		}
		if k < 1 {
			return Err(Error::WantedPayloadShardCountTooLow(k));
		}
		let k_po2 = next_lower_power_of_2(k);
		let n_po2 = next_higher_power_of_2(n);
		// If the coding rate of the power of 2 variants, is higher,
		// we would have to lower k by one order of magnitude base 2
		// which is true by definition
		assert!(n * k_po2 <= n_po2 * k);

		if n_po2 > FIELD_SIZE as usize {
			return Err(Error::WantedShardCountTooHigh(n));
		}
		Ok(Self { n: n_po2, k: k_po2, wanted_n: n })
	}

	// make a reed-solomon instance.
	pub fn make_encoder(&self) -> ReedSolomon {
		ReedSolomon::new(self.n, self.k, self.wanted_n)
			.expect("this struct is not created with invalid shard number; qed")
	}
}

pub struct ReedSolomon {
	n: usize,
	k: usize,
	wanted_n: usize,
}

impl ReedSolomon {
	/// Returns the size per shard in bytes
	pub fn shard_len(&self, payload_size: usize) -> usize {
		let payload_symbols = (payload_size + 1) / 2;
		let shard_symbols_ceil = (payload_symbols + self.k - 1) / self.k;
		let shard_bytes = shard_symbols_ceil * 2;
		shard_bytes
	}

	pub(crate) fn new(n: usize, k: usize, wanted_n: usize) -> Result<Self> {
		if !is_power_of_2(n) && !is_power_of_2(k) {
			Err(Error::ParamterMustBePowerOf2 { n, k })
		} else {
			Ok(Self { wanted_n, n, k })
		}
	}

	pub fn encode<S: Shard>(&self, bytes: &[u8]) -> Result<Vec<S>> {
		if bytes.is_empty() {
			return Err(Error::PayloadSizeIsZero);
		}

		// setup the shards, n is likely _larger_, so use the truely required number of shards

		// required shard length in bytes, rounded to full symbols
		let shard_len = self.shard_len(bytes.len());
		assert!(shard_len > 0);
		// collect all sub encoding runs

		let validator_count = self.wanted_n;
		let k2 = self.k * 2;
		// prepare one wrapped shard per validator
		let mut shards = vec![
			<S as From<Vec<u8>>>::from({
				let mut v = Vec::<u8>::with_capacity(shard_len);
				unsafe { v.set_len(shard_len) }
				v
			});
			validator_count
		];

		for (chunk_idx, i) in (0..bytes.len()).into_iter().step_by(k2).enumerate() {
			let end = std::cmp::min(i + k2, bytes.len());
			assert_ne!(i, end);
			let data_piece = &bytes[i..end];
			assert!(!data_piece.is_empty());
			assert!(data_piece.len() <= k2);
			let encoding_run = f2e16::encode_sub(data_piece, self.n, self.k)?;
			for val_idx in 0..validator_count {
				AsMut::<[[u8; 2]]>::as_mut(&mut shards[val_idx])[chunk_idx] = encoding_run[val_idx].0.to_be_bytes();
			}
		}

		Ok(shards)
	}

	/// each shard contains one symbol of one run of erasure coding
	pub fn reconstruct<S: Shard>(&self, received_shards: Vec<Option<S>>) -> Result<Vec<u8>> {
		let gap = self.n.saturating_sub(received_shards.len());

		let received_shards =
			received_shards.into_iter().take(self.n).chain(std::iter::repeat(None).take(gap)).collect::<Vec<_>>();

		assert_eq!(received_shards.len(), self.n);

		// must be collected after expanding `received_shards` to the anticipated size
		let mut existential_count = 0_usize;
		let erasures = received_shards
			.iter()
			.map(|x| x.is_none())
			.inspect(|erased| existential_count += !*erased as usize)
			.collect::<Vec<bool>>();

		if existential_count < self.k {
			return Err(Error::NeedMoreShards { have: existential_count, min: self.k, all: self.n });
		}

		// obtain a sample of a shard length and assume that is the truth
		let shard_len_in_syms = {
			let (first_shard_idx, first_shard_len) = received_shards
				.iter()
				.enumerate()
				.find_map(|(idx, shard)| {
					shard.as_ref().map(|shard| {
						let shard = AsRef::<[[u8; 2]]>::as_ref(shard);
						(idx, shard.len())
					})
				})
				.expect("Existential shard count is at least k shards. qed");

			// make sure all shards have the same length as the first one
			if let Some(other_shard_len) = received_shards[(first_shard_idx + 1)..].iter().find_map(|shard| {
				shard.as_ref().and_then(|shard| {
					let shard = AsRef::<[[u8; 2]]>::as_ref(shard);
					if first_shard_len != shard.len() {
						Some(shard.len())
					} else {
						None
					}
				})
			}) {
				return Err(Error::InconsistentShardLengths { first: first_shard_len, other: other_shard_len });
			}

			first_shard_len
		};

		// Evaluate error locator polynomial only once
		let mut error_poly_in_log = [Multiplier(0); FIELD_SIZE];
		f2e16::eval_error_polynomial(&erasures[..], &mut error_poly_in_log[..], FIELD_SIZE);

		let mut acc = Vec::<u8>::with_capacity(shard_len_in_syms * 2 * self.k);
		for i in 0..shard_len_in_syms {
			// take the i-th element of all shards and try to recover
			let decoding_run = received_shards
				.iter()
				.map(|x| {
					x.as_ref().map(|x| {
						let z = AsRef::<[[u8; 2]]>::as_ref(&x)[i];
						Additive(u16::from_be_bytes(z))
					})
				})
				.collect::<Vec<Option<Additive>>>();

			assert_eq!(decoding_run.len(), self.n);

			// reconstruct from one set of symbols which was spread over all erasure chunks
			let piece =
				f2e16::reconstruct_sub(&decoding_run[..], &erasures, self.n, self.k, &error_poly_in_log).unwrap();
			acc.extend_from_slice(&piece[..]);
		}

		Ok(acc)
	}
}

#[cfg(test)]
mod tests;
