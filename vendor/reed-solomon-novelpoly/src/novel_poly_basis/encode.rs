use super::*;

pub fn encode<S: Shard>(bytes: &[u8], validator_count: usize) -> Result<Vec<S>> {
	let params = CodeParams::derive_parameters(validator_count, recoverablity_subset_size(validator_count))?;

	let rs = params.make_encoder();
	rs.encode::<S>(bytes)
}
