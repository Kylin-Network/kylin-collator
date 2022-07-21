// A shard with a even number of elements, which can sliced into 2 byte haps
#[derive(Clone, Debug)]
pub struct WrappedShard {
	inner: Vec<u8>,
}

impl WrappedShard {
	/// Wrap `data`.
	pub fn new(mut data: Vec<u8>) -> Self {
		if data.len() & 0x01 == 0x01 {
			data.push(0);
		}

		WrappedShard { inner: data }
	}

	/// Unwrap and yield inner data.
	pub fn into_inner(self) -> Vec<u8> {
		self.inner
	}
}

impl From<Vec<u8>> for WrappedShard {
	fn from(data: Vec<u8>) -> Self {
		Self::new(data)
	}
}

impl AsRef<[u8]> for WrappedShard {
	fn as_ref(&self) -> &[u8] {
		self.inner.as_ref()
	}
}

impl AsMut<[u8]> for WrappedShard {
	fn as_mut(&mut self) -> &mut [u8] {
		self.inner.as_mut()
	}
}

impl AsRef<[[u8; 2]]> for WrappedShard {
	fn as_ref(&self) -> &[[u8; 2]] {
		assert_eq!(self.inner.len() & 0x01, 0);
		if self.inner.is_empty() {
			return &[];
		}
		unsafe { ::std::slice::from_raw_parts(&self.inner[0] as *const _ as _, self.inner.len() / 2) }
	}
}

impl AsMut<[[u8; 2]]> for WrappedShard {
	fn as_mut(&mut self) -> &mut [[u8; 2]] {
		let len = self.inner.len();
		assert_eq!(len & 0x01, 0);

		if self.inner.is_empty() {
			return &mut [];
		}
		unsafe { ::std::slice::from_raw_parts_mut(&mut self.inner[0] as *mut _ as _, len / 2) }
	}
}

impl std::iter::FromIterator<[u8; 2]> for WrappedShard {
	fn from_iter<I: IntoIterator<Item = [u8; 2]>>(iterable: I) -> Self {
		let iter = iterable.into_iter();

		let (l, _) = iter.size_hint();
		let mut inner = Vec::with_capacity(l * 2);

		for [a, b] in iter {
			inner.push(a);
			inner.push(b);
		}

		debug_assert_eq!(inner.len() & 0x01, 0);
		WrappedShard { inner }
	}
}
