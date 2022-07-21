pub const fn log2(mut x: usize) -> usize {
	let mut o: usize = 0;
	while x > 1 {
		x >>= 1;
		o += 1;
	}
	o
}

/// Check if x is a power of 2.
///
/// Zero is by definition not a power of 2.
pub const fn is_power_of_2(x: usize) -> bool {
	x > 0_usize && x & (x - 1) == 0
}

/// Tick up to the next higher power of 2 if
/// the provided number is not a power of 2.
pub const fn next_higher_power_of_2(k: usize) -> usize {
	if is_power_of_2(k) {
		k
	} else {
		1 << (log2(k) + 1)
	}
}

/// Tick down to the next higher power of 2 if
/// the provided number is not a power of 2.
pub const fn next_lower_power_of_2(k: usize) -> usize {
	if is_power_of_2(k) {
		k
	} else {
		1 << log2(k)
	}
}

/// Does not care about power of 2 requirements.
///
/// Covers the 1/3 case.
pub const fn recoverablity_subset_size(n_wanted_shards: usize) -> usize {
	(n_wanted_shards.saturating_sub(1) / 3) + 1
}

#[test]
fn three_f_plus_1() {
	assert_eq!(recoverablity_subset_size(0), 1); // degenerate
	assert_eq!(recoverablity_subset_size(1), 1);
	assert_eq!(recoverablity_subset_size(2), 1);
	assert_eq!(recoverablity_subset_size(3), 1);
	assert_eq!(recoverablity_subset_size(4), 2);
	assert_eq!(recoverablity_subset_size(5), 2);
	assert_eq!(recoverablity_subset_size(6), 2);
	assert_eq!(recoverablity_subset_size(8), 3);
	assert_eq!(recoverablity_subset_size(11), 4);

	assert_eq!(recoverablity_subset_size(173), 58);
	assert_eq!(recoverablity_subset_size(174), 58);
	assert_eq!(recoverablity_subset_size(175), 59);
}
