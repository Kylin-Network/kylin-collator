use super::*;

/// each shard contains one symbol of one run of erasure coding
pub fn reconstruct<'a, S: Shard>(received_shards: Vec<Option<S>>, validator_count: usize) -> Result<Vec<u8>> {
	let params = CodeParams::derive_parameters(validator_count, recoverablity_subset_size(validator_count))?;

	let rs = params.make_encoder();
	rs.reconstruct(received_shards)
}
