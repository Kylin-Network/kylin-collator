
pub fn reconstruct_sub(
	codewords: &[Option<Additive>],
	erasures: &[bool],
	n: usize,
	k: usize,
	error_poly: &[Multiplier; FIELD_SIZE],
) -> Result<Vec<u8>> {
	assert!(is_power_of_2(n), "Algorithm only works for 2^i sizes for N");
	assert!(is_power_of_2(k), "Algorithm only works for 2^i sizes for K");
	assert_eq!(codewords.len(), n);
	assert!(k <= n / 2);

	// the first k suffice for the original k message codewords
	let recover_up_to = k; // n;

	// The recovered _payload_ chunks AND parity chunks
	let mut recovered = vec![Additive(0); recover_up_to];

	// get rid of all `None`s
	let mut codeword = codewords
		.into_iter()
		.enumerate()
		.map(|(idx, sym)| {
			// fill the gaps with `0_u16` codewords
			if let Some(sym) = sym {
				(idx, *sym)
			} else {
				(idx, Additive(0))
			}
		})
		.map(|(idx, codeword)| {
			if idx < recovered.len() {
				recovered[idx] = codeword;
			}
			codeword
		})
		.collect::<Vec<Additive>>();

	// filled up the remaining spots with 0s
	assert_eq!(codeword.len(), n);

	//---------Erasure decoding----------------

	decode_main(&mut codeword[..], recover_up_to, &erasures[..], &error_poly[..], n);

	for idx in 0..recover_up_to {
		if erasures[idx] {
			recovered[idx] = codeword[idx];
		};
	}

	let mut recovered_bytes = Vec::with_capacity(recover_up_to * 2);
	recovered.into_iter().take(k).for_each(|x| recovered_bytes.extend_from_slice(&x.0.to_be_bytes()[..]));
	Ok(recovered_bytes)
}



/// recover determines how many shards to recover (starting from 0)
// technically we only need to recover
// the first `k` instead of all `n` which
// would include parity chunks.
pub(crate) fn decode_main(codeword: &mut [Additive], recover_up_to: usize, erasure: &[bool], log_walsh2: &[Multiplier], n: usize) {
	assert_eq!(codeword.len(), n);
	assert!(n >= recover_up_to);
	assert_eq!(erasure.len(), n);

	for i in 0..n {
		codeword[i] = if erasure[i] { Additive(0) } else { codeword[i].mul(log_walsh2[i]) };
	}

	inverse_afft(codeword, n, 0);

	tweaked_formal_derivative(codeword, n);

	afft(codeword, n, 0);

	for i in 0..recover_up_to {
		codeword[i] = if erasure[i] { codeword[i].mul(log_walsh2[i]) } else { Additive(0) };
	}
}


// Compute the evaluations of the error locator polynomial
// `fn decode_init`
// since this has only to be called once per reconstruction
pub fn eval_error_polynomial(erasure: &[bool], log_walsh2: &mut [Multiplier], n: usize) {
	let z = std::cmp::min(n, erasure.len());
	for i in 0..z {
		log_walsh2[i] = Multiplier(erasure[i] as Elt);
	}
	for i in z..n {
		log_walsh2[i] = Multiplier(0);
	}
	walsh(log_walsh2, FIELD_SIZE);
	for i in 0..n {
		let tmp = log_walsh2[i].to_wide() * LOG_WALSH[i].to_wide();
		log_walsh2[i] = Multiplier((tmp % ONEMASK as Wide) as Elt);
	}
	walsh(log_walsh2, FIELD_SIZE);
	for i in 0..z {
		if erasure[i] {
			log_walsh2[i] = Multiplier(ONEMASK) - log_walsh2[i];
		}
	}
}
